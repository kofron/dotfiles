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
    /// Org files or directories containing journal entries to scan.
    #[arg(required = true)]
    inputs: Vec<PathBuf>,
    /// Target date for the new journal entry. Defaults to today.
    #[arg(long)]
    date: Option<NaiveDate>,
    /// Write the resulting OrgFile JSON to this path instead of stdout.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Write the new entry into the resolved journal directory (auto-named YYYY-MM-DD.org).
    #[arg(long)]
    write: bool,
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
        template: template_path,
        inputs,
        date,
        output,
        write,
        emit,
    } = args;

    if write && output.is_some() {
        anyhow::bail!("--write cannot be combined with --output");
    }

    let parser = NomOrgParser;
    let mut template = parser
        .parse_file(&template_path)
        .with_context(|| format!("parsing template {:?}", template_path))?;

    let expanded = expand_inputs(&inputs, verbose)?;
    if expanded.is_empty() && verbose {
        eprintln!("warning: no Org files found in the provided inputs");
    }
    let mut journal_files = Vec::new();
    for path in &expanded {
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

    let (target_path, existed) = if write {
        let write_dir = resolve_write_directory(&inputs)
            .context("determining write directory for journal entry")?;
        let candidate = write_dir.join(format!("{date}.org"));
        let existed = candidate.exists();
        if existed {
            if verbose {
                eprintln!(
                    "Existing entry found at {:?}; using it as template",
                    candidate
                );
            }
            template = parser
                .parse_file(&candidate)
                .with_context(|| format!("parsing existing entry {:?}", candidate))?;
        }
        (Some(candidate), existed)
    } else {
        (None, false)
    };

    let new_entry = journal_new_entry_projector::build_from_files(
        &template,
        journal_files.iter(),
        date,
        verbose,
    );

    if let Some(target_path) = target_path {
        let text = format_org_file(&new_entry);
        fs::write(&target_path, text.as_bytes())
            .with_context(|| format!("writing {:?}", target_path))?;
        if existed {
            println!("Updated existing journal entry at {:?}", target_path);
        } else {
            println!("Wrote new journal entry to {:?}", target_path);
        }

        match emit {
            JournalOutputFormat::Json => {
                let json = serde_json::to_string_pretty(&new_entry)?;
                println!("{json}");
            }
            JournalOutputFormat::Org => {
                print!("{text}");
                if !text.ends_with('\n') {
                    println!();
                }
            }
        }
        return Ok(());
    }

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

fn resolve_write_directory(inputs: &[PathBuf]) -> Result<PathBuf> {
    if inputs.is_empty() {
        anyhow::bail!("no inputs provided to resolve write directory");
    }

    let mut dirs = Vec::new();
    for original in inputs {
        let canonical =
            fs::canonicalize(original).with_context(|| format!("resolving path {:?}", original))?;
        let metadata = fs::metadata(&canonical)
            .with_context(|| format!("reading metadata for {:?}", canonical))?;
        if metadata.is_dir() {
            dirs.push(canonical);
        } else if metadata.is_file() {
            if let Some(parent) = canonical.parent() {
                dirs.push(parent.to_path_buf());
            } else {
                anyhow::bail!("file {:?} has no parent directory", canonical);
            }
        } else {
            anyhow::bail!("{:?} is neither a file nor a directory", canonical);
        }
    }

    dirs.sort();
    dirs.dedup();
    if dirs.is_empty() {
        anyhow::bail!("failed to resolve candidate directories from inputs");
    }

    lowest_common_directory(&dirs).context("computing lowest common directory for journal inputs")
}

fn lowest_common_directory(paths: &[PathBuf]) -> Option<PathBuf> {
    let mut iter = paths.iter();
    let first = iter.next()?;
    let mut prefix = first.clone();
    for path in iter {
        while !path.starts_with(&prefix) {
            if !prefix.pop() {
                return None;
            }
        }
    }
    Some(prefix)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn resolve_write_directory_prefers_provided_directory() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let journal = tmp.path().join("journal");
        fs::create_dir_all(&journal).expect("mkdir journal");

        let resolved = resolve_write_directory(&[journal.clone()]).expect("resolve directory");

        assert_eq!(
            resolved,
            fs::canonicalize(&journal).expect("canonical fixed")
        );
    }

    #[test]
    fn resolve_write_directory_uses_lowest_common_parent_for_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let dir_a = root.join("a");
        let dir_b = root.join("b");
        fs::create_dir_all(&dir_a).expect("mkdir a");
        fs::create_dir_all(&dir_b).expect("mkdir b");

        let file_a = dir_a.join("alpha.org");
        let file_b = dir_b.join("beta.org");
        fs::write(&file_a, "* Alpha").expect("write alpha");
        fs::write(&file_b, "* Beta").expect("write beta");

        let resolved =
            resolve_write_directory(&[file_a.clone(), file_b.clone()]).expect("resolve files");

        assert_eq!(resolved, fs::canonicalize(root).expect("canonical root"));
    }

    #[test]
    fn lowest_common_directory_handles_nested_paths() {
        let paths: Vec<PathBuf> = [
            "work/journal/2025",
            "work/journal/2024",
            "work/journal/2025/wip",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();

        let lcd = lowest_common_directory(&paths).expect("lcd");
        assert_eq!(lcd, PathBuf::from("work/journal"));
    }
}
