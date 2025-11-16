use std::{
    collections::{BTreeSet, HashSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::{Args, Parser, Subcommand, ValueEnum};
use org::agenda::AgendaWhenKind;
use org::core::OrgFile;
use org::format_org_file;
use org::parser::NomOrgParser;
use org::projectors::agenda_projector::{self, ProjectOptions};
use org::projectors::journal_new_entry_projector;
use org::storage::OrgParser;

#[derive(Debug, Parser)]
#[command(
    name = "org",
    about = "Org-mode tooling built on the org crate",
    version
)]
struct Cli {
    /// Enable verbose logging for debugging.
    #[arg(long, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parse an Org file and print its structure.
    Parse(ParseArgs),

    /// Generate an agenda from one or more Org files.
    Agenda(AgendaArgs),

    /// Create a new journal entry by carrying forward incomplete TODOs.
    JournalNew(JournalNewArgs),

    /// Format an Org file, preserving untouched regions.
    Format(FormatArgs),
}

#[derive(Debug, Args)]
struct ParseArgs {
    /// Org files or directories containing Org files to parse.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    /// Emit JSON instead of a debug representation.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct AgendaArgs {
    /// Input Org files to include in the agenda.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    /// Inclusive start date filter (YYYY-MM-DD).
    #[arg(long)]
    from: Option<NaiveDate>,
    /// Inclusive end date filter (YYYY-MM-DD).
    #[arg(long)]
    to: Option<NaiveDate>,
    /// Emit JSON instead of a human-readable list.
    #[arg(long)]
    json: bool,
    /// Include undated TODO entries (agenda-style).
    #[arg(long)]
    include_todos: bool,
}

#[derive(Debug, Args)]
struct JournalNewArgs {
    /// Template Org file used as the base for the new entry.
    #[arg(long)]
    template: PathBuf,
    /// Directory containing journal entries to scan (parsed recursively).
    #[arg(long)]
    journal_dir: PathBuf,
    /// Target date for the new journal entry. Defaults to today.
    #[arg(long)]
    date: Option<NaiveDate>,
    /// Write the resulting OrgFile JSON to this path instead of stdout.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Output format (JSON or canonical Org text).
    #[arg(long, value_enum, default_value_t = JournalOutputFormat::Json)]
    emit: JournalOutputFormat,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum JournalOutputFormat {
    Json,
    Org,
}

#[derive(Debug, Args)]
struct FormatArgs {
    /// Org files or directories to format.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    /// Overwrite the file instead of printing to stdout.
    #[arg(long)]
    in_place: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let verbose = cli.verbose;
    match cli.command {
        Commands::Parse(args) => handle_parse(args, verbose),
        Commands::Agenda(args) => handle_agenda(args, verbose),
        Commands::JournalNew(args) => handle_journal_new(args, verbose),
        Commands::Format(args) => handle_format(args, verbose),
    }
}

fn handle_parse(args: ParseArgs, verbose: bool) -> Result<()> {
    let ParseArgs { inputs, json } = args;
    let expanded = expand_inputs(&inputs, verbose)?;
    if expanded.is_empty() {
        anyhow::bail!("no Org files found in the provided inputs");
    }

    let parser = NomOrgParser;
    let mut parsed = Vec::new();
    for path in expanded {
        if verbose {
            eprintln!("Parsing {:?}", path);
        }
        let file = parser
            .parse_file(&path)
            .with_context(|| format!("parsing {:?}", path))?;
        parsed.push((path, file));
    }

    if json {
        #[derive(serde::Serialize)]
        struct JsonOutput<'a> {
            path: String,
            org: &'a OrgFile,
        }

        let payload: Vec<JsonOutput<'_>> = parsed
            .iter()
            .map(|(path, file)| JsonOutput {
                path: path.display().to_string(),
                org: file,
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&payload)?);
    } else {
        for (idx, (path, file)) in parsed.iter().enumerate() {
            if parsed.len() > 1 {
                println!("== {} ==", path.display());
            }
            println!("{:#?}", file);
            if parsed.len() > 1 && idx + 1 < parsed.len() {
                println!();
            }
        }
    }
    Ok(())
}

fn handle_agenda(args: AgendaArgs, verbose: bool) -> Result<()> {
    let AgendaArgs {
        inputs,
        from,
        to,
        json,
        include_todos,
    } = args;

    let parser = NomOrgParser;
    let expanded = expand_inputs(&inputs, verbose)?;
    if expanded.is_empty() {
        anyhow::bail!("no Org files found in the provided inputs");
    }

    let mut files = Vec::new();
    for input in expanded {
        if verbose {
            eprintln!("Parsing agenda source {:?}", input);
        }
        let parsed = parser
            .parse_file(&input)
            .with_context(|| format!("parsing {:?}", input))?;
        files.push(parsed);
    }

    let mut items = agenda_projector::project_files_with_options(
        files.iter(),
        ProjectOptions { include_todos },
    );
    if let Some(from_date) = from {
        items.retain(|item| {
            matches!(item.when_kind, AgendaWhenKind::Todo) || item.span.start.date() >= from_date
        });
    }
    if let Some(to_date) = to {
        items.retain(|item| {
            matches!(item.when_kind, AgendaWhenKind::Todo) || item.span.start.date() <= to_date
        });
    }
    items.sort_by_key(|item| item.span.start);

    if items.is_empty() {
        eprintln!("No agenda items found for the provided inputs.");
        return Ok(());
    }

    if json {
        let json = serde_json::to_string_pretty(&items)?;
        println!("{json}");
    } else {
        for item in items {
            let date = item.span.start.date();
            let title = item.title;
            let kind = match item.when_kind {
                AgendaWhenKind::Scheduled => "SCHEDULED",
                AgendaWhenKind::Deadline => "DEADLINE",
                AgendaWhenKind::Timestamp => "TIMESTAMP",
                AgendaWhenKind::Closed => "CLOSED",
                AgendaWhenKind::Todo => "TODO",
            };
            let todo = item.todo.as_ref().map(|t| t.text.as_str()).unwrap_or("");
            let date_display = if matches!(item.when_kind, AgendaWhenKind::Todo) {
                "—".to_string()
            } else {
                date.to_string()
            };
            let tags = if item.tags.is_empty() {
                String::new()
            } else {
                format!(
                    " :{}:",
                    item.tags
                        .iter()
                        .map(|tag| tag.0.as_str())
                        .collect::<Vec<_>>()
                        .join(":")
                )
            };
            println!(
                "{} {:<10} {:<8} {}{}",
                date_display, kind, todo, title, tags
            );
        }
    }

    Ok(())
}

fn handle_journal_new(args: JournalNewArgs, verbose: bool) -> Result<()> {
    let JournalNewArgs {
        template,
        journal_dir,
        date,
        output,
        emit,
    } = args;

    let parser = NomOrgParser;
    let template = parser
        .parse_file(&template)
        .with_context(|| format!("parsing template {:?}", template))?;

    let journal_paths = collect_org_files(&journal_dir, verbose)
        .with_context(|| format!("scanning {:?}", journal_dir))?;
    let mut journal_files = Vec::new();
    for path in &journal_paths {
        match parser.parse_file(path) {
            Ok(file) => {
                if verbose {
                    eprintln!("Loaded journal file {:?}", path);
                }
                journal_files.push(file);
            }
            Err(err) => eprintln!("warning: failed to parse {:?}: {err:?}", path),
        }
    }

    let date = date.unwrap_or_else(|| Local::now().date_naive());
    let new_entry = journal_new_entry_projector::build_from_files(
        &template,
        journal_files.iter(),
        date,
        verbose,
    );

    match emit {
        JournalOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&new_entry)?;
            if let Some(path) = output {
                let mut file =
                    fs::File::create(&path).with_context(|| format!("creating {:?}", path))?;
                file.write_all(json.as_bytes())
                    .with_context(|| format!("writing {:?}", path))?;
                println!("Wrote new journal entry JSON to {:?}", path);
            } else {
                println!("{json}");
            }
        }
        JournalOutputFormat::Org => {
            let text = format_org_file(&new_entry);
            if let Some(path) = output {
                fs::write(&path, text.as_bytes()).with_context(|| format!("writing {:?}", path))?;
                println!("Wrote new journal entry to {:?}", path);
            } else {
                print!("{text}");
            }
        }
    }

    Ok(())
}

fn collect_org_files(dir: &Path, verbose: bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut visited = HashSet::new();
    visit_dir(dir, &mut out, &mut visited, verbose)?;
    out.sort();
    out.dedup();
    Ok(out)
}

fn handle_format(args: FormatArgs, verbose: bool) -> Result<()> {
    let FormatArgs { inputs, in_place } = args;
    let expanded = expand_inputs(&inputs, verbose)?;
    if expanded.is_empty() {
        anyhow::bail!("no Org files found in the provided inputs");
    }

    let parser = NomOrgParser;
    let mut first = true;

    for path in expanded {
        if verbose {
            eprintln!("Formatting {:?}", path);
        }
        let file = parser
            .parse_file(&path)
            .with_context(|| format!("parsing {:?}", path))?;
        let formatted = format_org_file(&file);

        if in_place {
            fs::write(&path, formatted.as_bytes())
                .with_context(|| format!("writing {:?}", path))?;
        } else {
            if !first {
                println!();
                println!("== {} ==", path.display());
            } else if inputs.len() > 1 {
                println!("== {} ==", path.display());
            }
            first = false;
            print!("{formatted}");
            if !formatted.ends_with('\n') {
                println!();
            }
        }
    }

    Ok(())
}

fn expand_inputs(paths: &[PathBuf], verbose: bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut visited = BTreeSet::new();
    for path in paths {
        let canonical =
            fs::canonicalize(path).with_context(|| format!("resolving path {:?}", path))?;
        let meta = fs::metadata(&canonical)
            .with_context(|| format!("reading metadata for {:?}", canonical))?;
        if meta.is_dir() {
            if verbose {
                eprintln!("Scanning directory {:?}", canonical);
            }
            for file in collect_org_files(&canonical, verbose)? {
                if visited.insert(file.clone()) {
                    out.push(file);
                }
            }
        } else if meta.is_file() {
            if canonical
                .extension()
                .map(|ext| ext == "org")
                .unwrap_or(false)
            {
                if verbose {
                    eprintln!("Adding file {:?}", canonical);
                }
                if visited.insert(canonical.clone()) {
                    out.push(canonical);
                }
            } else {
                anyhow::bail!("{:?} is not an .org file", canonical);
            }
        }
    }
    Ok(out)
}

fn visit_dir(
    path: &Path,
    out: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
    verbose: bool,
) -> Result<()> {
    let canonical = fs::canonicalize(path)?;
    if !visited.insert(canonical.clone()) {
        return Ok(());
    }

    let metadata = fs::metadata(&canonical)?;
    if metadata.is_dir() {
        if verbose {
            eprintln!("Visiting directory {:?}", canonical);
        }
        for entry in fs::read_dir(&canonical)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            visit_dir(&entry.path(), out, visited, verbose)?;
        }
    } else if metadata.is_file() {
        if canonical
            .extension()
            .map(|ext| ext == "org")
            .unwrap_or(false)
        {
            if verbose {
                eprintln!("Found org file {:?}", canonical);
            }
            out.push(canonical);
        }
    }
    Ok(())
}
