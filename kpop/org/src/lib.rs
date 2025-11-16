//! Org domain library generated from org/GUIDE.md modules.
//! Modules honor the guiding principles: keep the core pure, reuse existing designs,
//! and expose reusable projectors for higher-level workflows.

pub mod core {
    use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use std::{collections::BTreeSet, path::PathBuf};
    use uuid::Uuid;

    /* ------------------------------- IDs ------------------------------- */

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct OrgFileId(pub Uuid);

    impl OrgFileId {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct HeadingId(pub Uuid);

    impl HeadingId {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }

    /* ------------------------------ Aggregate ------------------------------ */

    /// Aggregate root: a single `.org` file.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OrgFile {
        pub id: OrgFileId,
        /// Optional filesystem path if the file originates from disk.
        pub path: Option<PathBuf>,

        /// Title from `#+title:` (if present).
        pub title: Option<String>,

        /// File-wide tags from `#+filetags:` (normalized to a set).
        #[serde(default)]
        pub file_tags: BTreeSet<Tag>,

        /// File-local settings that influence semantics (TODO sequences, priorities, etc.).
        #[serde(default)]
        pub settings: FileSettings,

        /// Content before the first heading (preamble).
        #[serde(default)]
        pub preamble: Vec<BlockWithSource>,

        /// Top-level headings.
        #[serde(default)]
        pub headings: Vec<Heading>,

        /// Original source text captured during parsing for round-trip formatting.
        #[serde(skip_serializing, skip_deserializing)]
        pub source_text: Option<String>,
    }

    impl OrgFile {
        pub fn new(path: Option<PathBuf>) -> Self {
            Self {
                id: OrgFileId::new(),
                path,
                title: None,
                file_tags: BTreeSet::new(),
                settings: FileSettings::default(),
                preamble: vec![],
                headings: vec![],
                source_text: None,
            }
        }
    }

    /* ------------------------------ Entities ------------------------------ */

    /// A heading node with a section and children (Org tree).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Heading {
        pub id: HeadingId,
        /// 1..=8 in Org; invariant is not enforced at type level but should be validated.
        pub level: u8,

        /// Title as inline rich text.
        pub title: RichText,

        /// Optional TODO keyword (e.g., TODO, NEXT, WAIT, DONE).
        pub todo: Option<TodoKeyword>,

        /// Optional single-letter priority like [#A].
        pub priority: Option<Priority>,

        /// Tags after the headline (`:tag1:tag2:`).
        #[serde(default)]
        pub tags: BTreeSet<Tag>,

        /// Planning line(s): SCHEDULED, DEADLINE, CLOSED.
        #[serde(default)]
        pub planning: Planning,

        /// Property drawer (key/value string pairs).
        #[serde(default)]
        pub properties: PropertyDrawer,

        /// Logbook (CLOCK entries + state change notes).
        #[serde(default)]
        pub logbook: Logbook,

        /// The section (content under this headline until the next heading).
        #[serde(default)]
        pub section: Section,

        /// Child headings.
        #[serde(default)]
        pub children: Vec<Heading>,

        /// Optional unique CUSTOM_ID or ID property resolved for cross-links.
        pub canonical_id: Option<String>,

        /// Captured headline source range (used when formatting if untouched).
        #[serde(skip_serializing, skip_deserializing)]
        pub headline_range: Option<SourceRange>,

        /// Planning lines source range.
        #[serde(skip_serializing, skip_deserializing)]
        pub planning_range: Option<SourceRange>,

        /// Property drawer source range.
        #[serde(skip_serializing, skip_deserializing)]
        pub properties_range: Option<SourceRange>,

        /// Logbook drawer source range.
        #[serde(skip_serializing, skip_deserializing)]
        pub logbook_range: Option<SourceRange>,
    }

    impl Heading {
        pub fn new(level: u8, title: RichText) -> Self {
            Self {
                id: HeadingId::new(),
                level,
                title,
                todo: None,
                priority: None,
                tags: BTreeSet::new(),
                planning: Planning::default(),
                properties: PropertyDrawer::default(),
                logbook: Logbook::default(),
                section: Section::default(),
                children: vec![],
                canonical_id: None,
                headline_range: None,
                planning_range: None,
                properties_range: None,
                logbook_range: None,
            }
        }

        pub fn mark_headline_dirty(&mut self) {
            self.headline_range = None;
        }

        pub fn mark_planning_dirty(&mut self) {
            self.planning_range = None;
        }

        pub fn mark_properties_dirty(&mut self) {
            self.properties_range = None;
        }

        pub fn mark_logbook_dirty(&mut self) {
            self.logbook_range = None;
        }
    }

    /* ----------------------------- File settings ----------------------------- */

    /// File-local settings that influence parsing/semantics (a minimal useful subset).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FileSettings {
        /// Ordered TODO sequences; the last state in a sequence can be a done-type state.
        /// Example: [["TODO","NEXT","WAIT","|","DONE","CANCELLED"]]
        #[serde(default)]
        pub todo_sequences: Vec<TodoSequence>,

        /// Recognized priorities (default Org is A..C).
        #[serde(default = "FileSettings::default_priorities")]
        pub priorities: Vec<Priority>,

        /// Default time zone for timestamps when not explicit.
        #[serde(with = "serde_fixed_offset_opt")]
        pub default_tz: Option<FixedOffset>,

        /// Any other per-file key/values from #+KEY: VALUE lines.
        #[serde(default)]
        pub meta: IndexMap<String, String>,
    }

    impl Default for FileSettings {
        fn default() -> Self {
            Self {
                todo_sequences: vec![],
                priorities: Self::default_priorities(),
                default_tz: None,
                meta: IndexMap::new(),
            }
        }
    }

    impl FileSettings {
        fn default_priorities() -> Vec<Priority> {
            vec![Priority('A'), Priority('B'), Priority('C')]
        }
    }

    /// TODO sequence definition; `|` splits undone/done sets in Org.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TodoSequence {
        /// The sequence items in order; include a literal "|" to mark divider.
        pub items: Vec<String>,
    }

    /* ---------------------------- Value Objects ---------------------------- */

    /// Tag wrapper (normalized to lowercase for equality/ordering, but we keep original for display).
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Tag(pub String);

    impl From<&str> for Tag {
        fn from(s: &str) -> Self {
            Self(s.to_string())
        }
    }

    /// Single-letter priority, e.g. [#A].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct Priority(pub char);

    /// Todo keyword with a "done" flag so we can respect file-specific vocabularies.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TodoKeyword {
        pub text: String,
        pub is_done: bool,
    }

    /// Planning line(s): SCHEDULED, DEADLINE, CLOSED.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Planning {
        pub scheduled: Option<Timestamp>,
        pub deadline: Option<Timestamp>,
        pub closed: Option<Timestamp>,
    }

    /// A timestamp with optional time, range, repeater, and delay.
    ///
    /// Supports active `<...>` and inactive `[...]` timestamps. For agenda usage,
    /// normalize to a `TimeSpan` with a start (and optional end).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Timestamp {
        /// Whether the timestamp is "active" (`<...>`) or inactive (`[...]`).
        pub active: bool,

        /// Date/time portion. If `time` is None, it's an all-day date.
        pub date: NaiveDate,
        pub time: Option<NaiveTime>,

        /// Optional explicit zone; falls back to file.default_tz or local policy.
        #[serde(with = "serde_fixed_offset_opt")]
        pub tz: Option<FixedOffset>,

        /// Optional range end (same date if omitted but end_time present).
        pub end: Option<TimestampEnd>,

        /// Optional repeater cookie (`+1w`, `++1m`, `.+2d`).
        pub repeater: Option<Repeater>,

        /// Optional delay/warning cookie (`-2d`, `-1w`, etc.).
        pub delay: Option<Delay>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TimestampEnd {
        pub date: Option<NaiveDate>, // if None, same date as start
        pub time: Option<NaiveTime>, // range of times on the same date
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Repeater {
        pub kind: RepeaterKind,
        pub interval: DateOffset,
    }

    /// `+` (from last closed), `++` (from base), `.+` (from now).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum RepeaterKind {
        FromLast, // `+`
        FromBase, // `++`
        FromNow,  // `.+`
    }

    /// Delay/warning cookie such as `-2d`.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Delay {
        pub before: bool, // currently Org supports "before" warnings, keep extensible
        pub offset: DateOffset,
    }

    /// A calendar offset in calendar units (weeks, months, etc.) — not just seconds.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct DateOffset {
        pub years: i32,
        pub months: i32,
        pub weeks: i32,
        pub days: i32,
        pub hours: i32,
        pub minutes: i32,
    }

    impl DateOffset {
        pub fn weeks(w: i32) -> Self {
            Self {
                years: 0,
                months: 0,
                weeks: w,
                days: 0,
                hours: 0,
                minutes: 0,
            }
        }
        pub fn days(d: i32) -> Self {
            Self {
                years: 0,
                months: 0,
                weeks: 0,
                days: d,
                hours: 0,
                minutes: 0,
            }
        }
    }

    mod serde_fixed_offset_opt {
        use chrono::FixedOffset;
        use serde::{Deserialize, Deserializer, Serializer};

        pub fn serialize<S>(value: &Option<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match value {
                Some(offset) => serializer.serialize_some(&offset.local_minus_utc()),
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<FixedOffset>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let opt = Option::<i32>::deserialize(deserializer)?;
            Ok(opt.and_then(FixedOffset::east_opt))
        }
    }

    /// A normalized, fully-resolved time span for agenda calculations.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TimeSpan {
        pub start: NaiveDateTime,
        pub end: Option<NaiveDateTime>,
    }

    /* ---------------------------- Content Model ---------------------------- */

    /// Section content under a headline.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Section {
        #[serde(default)]
        pub blocks: Vec<BlockWithSource>,
    }

    /// Block-level elements. `Unknown` preserves round-trippability.
    #[non_exhaustive]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Block {
        Paragraph(RichText),
        List(List),
        Quote(Vec<Block>),
        Example {
            raw: String,
        },
        SrcBlock(SrcBlock),
        Drawer(Drawer),
        Table(Table),
        HorizontalRule,
        Comment(String),
        Directive {
            key: String,
            value: String,
        },
        /// For constructs we don’t parse yet; `kind` might be "LATEX" or similar.
        Unknown {
            kind: String,
            raw: String,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SourceRange {
        pub start: usize,
        pub end: usize,
    }

    impl SourceRange {
        pub fn slice<'a>(&self, source: &'a str) -> &'a str {
            &source[self.start..self.end]
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct BlockWithSource {
        pub block: Block,
        #[serde(skip_serializing, skip_deserializing)]
        pub source: Option<SourceRange>,
    }

    impl BlockWithSource {
        pub fn new(block: Block) -> Self {
            Self {
                block,
                source: None,
            }
        }

        pub fn from_source(block: Block, source: SourceRange) -> Self {
            Self {
                block,
                source: Some(source),
            }
        }

        /// Marks the block as modified, clearing stored raw text.
        pub fn mark_dirty(&mut self) {
            self.source = None;
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Drawer {
        pub name: String, // e.g., "PROPERTIES" handled separately, but this allows custom drawers too.
        pub content: Vec<Block>,
    }

    /// Property drawer — canonical location is under a heading; we keep it typed.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct PropertyDrawer {
        #[serde(default)]
        pub props: IndexMap<String, String>,
    }

    /// Logbook captures CLOCK entries and state-change notes.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Logbook {
        #[serde(default)]
        pub clock: Vec<ClockEntry>,
        #[serde(default)]
        pub state_changes: Vec<StateChange>,
        /// Any raw lines unknown to the model, preserved for round-trip.
        #[serde(default)]
        pub raw: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ClockEntry {
        pub start: Timestamp,
        pub end: Option<Timestamp>,
        /// If present, the duration parsed from the => part (kept as minutes).
        pub minutes: Option<i64>,
        /// Original raw line for fidelity (optional).
        pub raw: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct StateChange {
        pub from: Option<TodoKeyword>,
        pub to: Option<TodoKeyword>,
        pub at: Option<Timestamp>,
        pub note: Option<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SrcBlock {
        pub language: Option<String>,
        pub parameters: IndexMap<String, String>,
        pub code: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Table {
        /// Raw lines are preserved for full fidelity; optional structured cells can be added later.
        pub raw: Vec<String>,
    }

    /// A rich-text run used for headlines and paragraphs.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct RichText {
        #[serde(default)]
        pub inlines: Vec<Inline>,
    }

    #[non_exhaustive]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Inline {
        Text(String),
        Emphasis {
            kind: Emphasis,
            children: Vec<Inline>,
        },
        Code(String),
        Verbatim(String),
        Link(Link),
        Target(String),      // <<target>>
        FootnoteRef(String), // [fn:1]
        Entity(String),      // \alpha, &mdash;, etc.
        // Unknown / extension points
        Unknown {
            kind: String,
            raw: String,
        },
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Emphasis {
        Bold,
        Italic,
        Underline,
        Strike,
        Mark,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Link {
        pub kind: LinkKind,
        pub desc: Option<Vec<Inline>>,
    }

    #[non_exhaustive]
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum LinkKind {
        File {
            path: String,
            search: Option<String>,
        }, // file:path::search
        Http {
            url: String,
        },
        Id {
            id: String,
        }, // id:custom-id
        Custom {
            protocol: String,
            target: String,
        }, // e.g., mailto: user@host
    }

    /// A list (ordered/unordered/description) with optional checkboxes.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct List {
        pub kind: ListKind,
        pub items: Vec<ListItem>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ListKind {
        Unordered,
        Ordered,
        Description,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ListItem {
        /// For description lists, this is the "term".
        pub label: Option<RichText>,
        pub content: Vec<Block>,
        pub checkbox: Option<Checkbox>,
        pub counter: Option<i64>, // for ordered lists
        pub tags: BTreeSet<Tag>,  // e.g., `:foo:bar:` trailing on bullet
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Checkbox {
        Empty,   // [ ]
        Partial, // [-]
        Checked, // [X]
    }

    /* ---------------------------- Errors (domain) ---------------------------- */

    #[derive(Debug, thiserror::Error)]
    pub enum DomainError {
        #[error("heading level {0} is out of bounds (1..=8)")]
        InvalidLevel(u8),
        #[error("duplicate heading id")]
        DuplicateHeadingId,
        #[error("invalid timestamp")]
        InvalidTimestamp,
    }

    /* ----------------------- Utility: Title text extraction ----------------------- */

    impl RichText {
        /// Render a plain text approximation (useful for agenda titles).
        pub fn plain_text(&self) -> String {
            fn rec(xs: &[Inline], out: &mut String) {
                for x in xs {
                    match x {
                        Inline::Text(t) => out.push_str(t),
                        Inline::Emphasis { children, .. } => rec(children, out),
                        Inline::Code(t) | Inline::Verbatim(t) => out.push_str(t),
                        Inline::Link(Link { desc: Some(d), .. }) => rec(d, out),
                        Inline::Link(Link { desc: None, kind }) => match kind {
                            LinkKind::Http { url } => out.push_str(url),
                            LinkKind::File { path, .. } => out.push_str(path),
                            LinkKind::Id { id } => out.push_str(id),
                            LinkKind::Custom { protocol, target } => {
                                out.push_str(protocol);
                                out.push(':');
                                out.push_str(target);
                            }
                        },
                        Inline::Target(t) | Inline::FootnoteRef(t) | Inline::Entity(t) => {
                            out.push_str(t)
                        }
                        Inline::Unknown { raw, .. } => out.push_str(raw),
                    }
                }
            }
            let mut s = String::new();
            rec(&self.inlines, &mut s);
            s
        }
    }
}

pub mod journal {
    //! Read-model helpers for org-journal style workflows.
    //!
    //! This does not prescribe a specific folder layout. It provides types to index
    //! entries by date when your project uses headings-as-entries.

    use super::core::*;
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    /// Reference to a heading inside a file.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct NodeRef {
        pub file_id: OrgFileId,
        pub heading_id: HeadingId,
    }

    /// Journal key: a date bucket (e.g., daily).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
    pub struct JournalKey {
        pub date: NaiveDate,
    }

    /// Entry reference enriched with display data for views.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct JournalEntryRef {
        pub key: JournalKey,
        pub node: NodeRef,
        pub title: String,
        pub tags: Vec<Tag>,
    }

    /// An index from date → entries, computed from one or more Org files.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct JournalIndex {
        pub entries: BTreeMap<JournalKey, Vec<JournalEntryRef>>,
    }

    impl JournalIndex {
        pub fn add(&mut self, key: JournalKey, entry: JournalEntryRef) {
            self.entries.entry(key).or_default().push(entry);
        }
    }
}

pub mod agenda {
    //! Read-model helpers for agenda/planning. These structs are projections built
    //! from `core` and intended for scheduling views, queries, and sorting.

    use super::core::*;
    use chrono::{NaiveDate, NaiveDateTime};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum AgendaWhenKind {
        Scheduled,
        Deadline,
        Timestamp, // a timestamp found in body or headline
        Closed,
        Todo,
    }

    /// Agenda item is a denormalized slice useful for agenda lists.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AgendaItem {
        pub id: uuid::Uuid,
        pub source_file: OrgFileId,
        pub source_heading: HeadingId,

        pub when_kind: AgendaWhenKind,
        pub span: TimeSpan, // normalized start/end
        pub active: bool,   // from timestamp
        pub title: String,  // plain-text title
        pub todo: Option<TodoKeyword>,
        pub priority: Option<Priority>,
        pub tags: Vec<Tag>,
        pub context_path: Vec<String>, // heading path for display/breadcrumbs
    }

    impl AgendaItem {
        pub fn new(
            source_file: OrgFileId,
            source_heading: HeadingId,
            when_kind: AgendaWhenKind,
            span: TimeSpan,
            active: bool,
            title: String,
            todo: Option<TodoKeyword>,
            priority: Option<Priority>,
            tags: Vec<Tag>,
            context_path: Vec<String>,
        ) -> Self {
            Self {
                id: uuid::Uuid::new_v4(),
                source_file,
                source_heading,
                when_kind,
                span,
                active,
                title,
                todo,
                priority,
                tags,
                context_path,
            }
        }
    }

    /// A convenience filter useful for producing multi-day agendas.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AgendaRange {
        pub from: NaiveDate,
        pub to: NaiveDate, // inclusive
    }

    impl AgendaRange {
        pub fn contains(&self, dt: NaiveDateTime) -> bool {
            let start = self.from.and_hms_opt(0, 0, 0).unwrap();
            let end = self.to.and_hms_opt(23, 59, 59).unwrap();
            dt >= start && dt <= end
        }
    }
}

pub mod storage {
    use super::core::OrgFile;
    use super::workspace::{OrgWorkspace, RelPath, ScanPolicy};
    use anyhow::Result;
    use std::path::Path;

    /// Builds a workspace tree by scanning the filesystem.
    pub trait WorkspaceRepository {
        /// Scan `root_dir` according to `policy`, returning a workspace with `Stub` file entries.
        fn scan(&self, root_dir: &Path, policy: &ScanPolicy) -> Result<OrgWorkspace>;

        /// Parse and hydrate a single file in the workspace (idempotent).
        fn load_file(&self, ws: &mut OrgWorkspace, rel_path: &RelPath) -> Result<()>;

        /// Persist any workspace-level cache/index you maintain (optional).
        fn save_cache(&self, ws_cache_path: &Path, ws: &OrgWorkspace) -> Result<()>;

        /// Load a previously saved cache/index (optional).
        fn load_cache(&self, ws_cache_path: &Path) -> Result<OrgWorkspace>;
    }

    /// If you want separation of concerns: parsing is independent of scanning.
    pub trait OrgParser {
        fn parse_file(&self, abs_path: &Path) -> Result<OrgFile>;
    }
}

pub mod workspace {
    //! Workspace (directory tree) aggregate that contains Org files.
    //!
    //! DDD sketch:
    //! - Aggregate root: OrgWorkspace
    //! - Entities: Folder (Dir), OrgFileEntry
    //! - Value objects: RelPath, FileStats, ScanPolicy, WorkspaceIndexes
    //!
    //! Notes:
    //! - Files are separate aggregates (`core::OrgFile`); the workspace holds references and
    //!   *optionally* the parsed content (lazy load).
    //! - Every path is stored relative to the workspace root (`RelPath`), while the root path
    //!   on disk lives in `OrgWorkspace::root_abs`.
    //! - `WorkspaceIndexes` is optional and can be built by your application layer.

    use super::core::{HeadingId, OrgFile, OrgFileId, Tag};
    use chrono::{DateTime, Utc};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::PathBuf,
    };
    use uuid::Uuid;

    /* ------------------------------- IDs ------------------------------- */

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct WorkspaceId(pub Uuid);

    impl WorkspaceId {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct FolderId(pub Uuid);

    impl FolderId {
        pub fn new() -> Self {
            Self(Uuid::new_v4())
        }
    }

    /* ---------------------------- Value Objects ---------------------------- */

    /// A POSIX-like relative path from the workspace root (no leading '/').
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct RelPath(pub String);

    impl RelPath {
        pub fn root() -> Self {
            Self("".into())
        }
        pub fn join(&self, segment: &str) -> Self {
            if self.0.is_empty() {
                Self(segment.to_string())
            } else {
                Self(format!("{}/{}", self.0, segment))
            }
        }
        pub fn parent(&self) -> Option<Self> {
            if self.0.is_empty() {
                None
            } else {
                let mut parts = self.0.split('/').collect::<Vec<_>>();
                parts.pop();
                Some(Self(parts.join("/")))
            }
        }
        pub fn file_name(&self) -> Option<&str> {
            if self.0.is_empty() {
                None
            } else {
                self.0.rsplit('/').next()
            }
        }
    }

    /// File metadata we can capture without parsing the file.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct FileStats {
        pub size_bytes: Option<u64>,
        pub modified_utc: Option<DateTime<Utc>>,
        pub is_symlink: bool,
    }

    /// Scanning rules (infra reads these; model persists them for reproducibility).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ScanPolicy {
        /// Glob patterns (workspace-relative) to ignore, e.g., `**/.git/**`, `**/*.org_archive`.
        #[serde(default)]
        pub ignore_globs: Vec<String>,
        /// Only include files matching these globs; if empty, defaults to `**/*.org`.
        #[serde(default)]
        pub include_globs: Vec<String>,
        /// Whether to follow symlinks while scanning.
        #[serde(default)]
        pub follow_symlinks: bool,
    }

    impl Default for ScanPolicy {
        fn default() -> Self {
            Self {
                ignore_globs: vec![
                    "**/.git/**".into(),
                    "**/.direnv/**".into(),
                    "**/target/**".into(),
                ],
                include_globs: vec!["**/*.org".into()],
                follow_symlinks: false,
            }
        }
    }

    /* ----------------------------- File entries ----------------------------- */

    /// Whether the file content has been loaded (parsed) into memory.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum FileContent {
        /// Only metadata is present; content can be loaded on demand.
        Stub,
        /// Parsed content is present.
        Loaded(Box<OrgFile>),
    }

    /// An Org file inside a folder. Points at the `core::OrgFile` aggregate by ID.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OrgFileEntry {
        /// Stable ID of the underlying `OrgFile` aggregate.
        pub file_id: OrgFileId,
        /// Relative path (from workspace root) to this file, e.g., `journal/2025-11-15.org`.
        pub rel_path: RelPath,
        /// Convenient handle: just the file name (stem + extension).
        pub file_name: String,
        /// Metadata pulled from the filesystem.
        pub stats: FileStats,
        /// Optional title extracted from the file (if we read the preamble cheaply).
        pub title_hint: Option<String>,
        /// Optional file-level tags (from #+filetags) cached for quick filtering.
        #[serde(default)]
        pub file_tags: BTreeSet<Tag>,
        /// In-memory content state.
        pub content: FileContent,
    }

    impl OrgFileEntry {
        pub fn is_loaded(&self) -> bool {
            matches!(self.content, FileContent::Loaded(_))
        }
        pub fn loaded(&self) -> Option<&OrgFile> {
            match &self.content {
                FileContent::Loaded(x) => Some(x),
                FileContent::Stub => None,
            }
        }
        pub fn loaded_mut(&mut self) -> Option<&mut OrgFile> {
            match &mut self.content {
                FileContent::Loaded(x) => Some(x),
                FileContent::Stub => None,
            }
        }
    }

    /* -------------------------------- Folders -------------------------------- */

    /// A folder (directory) that can contain subfolders and Org files.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Folder {
        pub id: FolderId,
        /// The folder name (last path component). Root may be empty.
        pub name: String,
        /// Path relative to workspace root.
        pub rel_path: RelPath,
        /// Org files directly contained in this folder (no nesting).
        #[serde(default)]
        pub files: Vec<OrgFileEntry>,
        /// Child folders (entities).
        #[serde(default)]
        pub subdirs: Vec<Folder>,
        /// Arbitrary per-folder metadata (e.g., display order).
        #[serde(default)]
        pub meta: IndexMap<String, String>,
    }

    impl Folder {
        pub fn new_root() -> Self {
            Self {
                id: FolderId::new(),
                name: String::new(),
                rel_path: RelPath::root(),
                files: vec![],
                subdirs: vec![],
                meta: IndexMap::new(),
            }
        }

        pub fn new_child(parent: &RelPath, name: String) -> Self {
            Self {
                id: FolderId::new(),
                name: name.clone(),
                rel_path: parent.join(&name),
                files: vec![],
                subdirs: vec![],
                meta: IndexMap::new(),
            }
        }

        /// Depth-first iterator over all descendant folders (including self).
        pub fn walk<'a>(&'a self, out: &mut Vec<&'a Folder>) {
            out.push(self);
            for d in &self.subdirs {
                d.walk(out);
            }
        }

        /// Find a subfolder by relative path.
        pub fn find_dir<'a>(&'a self, rel: &RelPath) -> Option<&'a Folder> {
            if &self.rel_path == rel {
                return Some(self);
            }
            for d in &self.subdirs {
                if let Some(hit) = d.find_dir(rel) {
                    return Some(hit);
                }
            }
            None
        }

        /// Collect all Org file entries recursively.
        pub fn collect_files<'a>(&'a self, out: &mut Vec<&'a OrgFileEntry>) {
            for f in &self.files {
                out.push(f);
            }
            for d in &self.subdirs {
                d.collect_files(out);
            }
        }
    }

    /* ----------------------------- Workspace root ----------------------------- */

    /// Aggregate root representing the directory tree on disk.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct OrgWorkspace {
        pub id: WorkspaceId,
        /// Absolute path of the workspace root on disk.
        pub root_abs: PathBuf,
        /// Root folder entity (its `rel_path` is empty).
        pub root: Folder,
        /// How this workspace was scanned.
        pub scan_policy: ScanPolicy,
        /// Optional cross-file indexes for fast queries (kept minimal here).
        #[serde(default)]
        pub indexes: WorkspaceIndexes,
    }

    impl OrgWorkspace {
        pub fn new(root_abs: PathBuf) -> Self {
            Self {
                id: WorkspaceId::new(),
                root_abs,
                root: Folder::new_root(),
                scan_policy: ScanPolicy::default(),
                indexes: WorkspaceIndexes::default(),
            }
        }

        /// Helper to resolve a relative path to an absolute on-disk path.
        pub fn abs_path(&self, rel: &RelPath) -> PathBuf {
            if rel.0.is_empty() {
                self.root_abs.clone()
            } else {
                self.root_abs.join(&rel.0)
            }
        }

        /// Snapshot all files in depth-first order.
        pub fn all_files(&self) -> Vec<&OrgFileEntry> {
            let mut v = Vec::new();
            self.root.collect_files(&mut v);
            v
        }

        /// Find a file entry by its `OrgFileId`.
        pub fn find_file_by_id(&self, id: OrgFileId) -> Option<&OrgFileEntry> {
            self.all_files().into_iter().find(|f| f.file_id == id)
        }

        /// (Re)build lightweight path index; heavier indexes belong to application layer.
        pub fn rebuild_path_index(&mut self) {
            self.indexes.files_by_relpath.clear();
            fn rec(idx: &mut IndexMap<RelPath, OrgFileId>, folder: &Folder) {
                for f in &folder.files {
                    idx.insert(f.rel_path.clone(), f.file_id);
                }
                for d in &folder.subdirs {
                    rec(idx, d);
                }
            }
            rec(&mut self.indexes.files_by_relpath, &self.root);
        }
    }

    /* -------------------------------- Indexes -------------------------------- */

    /// Minimal, optional indexes across the workspace.
    #[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct WorkspaceIndexes {
        /// Fast lookup: relpath → OrgFileId.
        #[serde(default)]
        pub files_by_relpath: IndexMap<RelPath, OrgFileId>,

        /// CUSTOM_ID/ID → (file, heading). Fill this from parsed content if you need it.
        #[serde(default)]
        pub id_index: BTreeMap<String, (OrgFileId, HeadingId)>,

        /// Tag → list of (file, heading). Useful for xref and agenda filters.
        #[serde(default)]
        pub tag_index: BTreeMap<Tag, Vec<(OrgFileId, HeadingId)>>,
    }

    /* ----------------------------- Constructors ------------------------------ */

    pub fn make_file_entry(
        root: &OrgWorkspace,
        rel: RelPath,
        stats: FileStats,
        title_hint: Option<String>,
        file_tags: BTreeSet<Tag>,
        content: Option<OrgFile>,
    ) -> OrgFileEntry {
        // If content is present, use its id; otherwise assign a deterministic new id.
        let (file_id, content_state) = match content {
            Some(org) => (org.id, FileContent::Loaded(Box::new(org))),
            None => (OrgFileId(Uuid::new_v4()), FileContent::Stub),
        };

        let file_name = rel.file_name().unwrap_or_default().to_string();
        let _abs = root.abs_path(&rel);
        OrgFileEntry {
            file_id,
            rel_path: rel,
            file_name,
            stats,
            title_hint,
            file_tags,
            content: content_state,
        }
    }
}

pub mod parser {
    //! Minimal Org parser built on `nom`.
    //!
    //! Goals: correctness-first structure, easy to extend, preserves round-trip via Unknown/raw fields.
    //! Parsing strategy:
    //! - Top-level scan is line-oriented and stack-builds the heading tree by levels (`*`, `**`, ...).
    //! - Each *headline* is parsed with `nom` combinators (TODO, priority, title, tags).
    //! - Under a headline, we parse planning lines, known drawers, and then section blocks until the next headline.

    use crate::core::*;
    use crate::storage::OrgParser;
    use anyhow::{Context, Result, anyhow};
    use chrono::{NaiveDate, NaiveTime};
    use nom::{
        IResult,
        branch::alt,
        bytes::complete::{is_not, tag, take_till1, take_until, take_while, take_while1},
        character::complete::{
            anychar, char, digit1, line_ending, not_line_ending, space0, space1,
        },
        combinator::{map, map_res, opt, recognize},
        error::{VerboseError, VerboseErrorKind},
        multi::{many0, many1},
        sequence::{delimited, preceded, terminated, tuple},
    };
    use std::{collections::BTreeSet, fs, path::Path, path::PathBuf};

    /* ------------------------ Public entry points ------------------------ */

    /// Parse an Org document from a string.
    pub fn parse_org_from_str(path: Option<PathBuf>, input: &str) -> Result<OrgFile> {
        let base_len = input.len();

        // 1) File metadata & preamble (before first heading).
        let (rest, (settings, file_title, file_tags, preamble_blocks)) =
            parse_preamble(input, base_len).map_err(to_anyhow("preamble"))?;

        let mut file = OrgFile::new(path);
        file.source_text = Some(input.to_string());
        file.title = file_title;
        file.file_tags = file_tags.into_iter().collect();
        file.settings = settings;
        file.preamble = preamble_blocks;

        // 2) Headings (stack build).
        let (_rest, headings) =
            parse_headings_tree(rest, base_len).map_err(to_anyhow("headings"))?;
        file.headings = headings;

        Ok(file)
    }

    /// Concrete parser implementing the `storage::OrgParser` trait.
    pub struct NomOrgParser;

    impl OrgParser for NomOrgParser {
        fn parse_file(&self, abs_path: &Path) -> Result<OrgFile> {
            let text =
                fs::read_to_string(abs_path).with_context(|| format!("reading {:?}", abs_path))?;
            parse_org_from_str(Some(abs_path.to_path_buf()), &text)
        }
    }

    type PResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

    fn to_anyhow(label: &'static str) -> impl Fn(nom::Err<VerboseError<&str>>) -> anyhow::Error {
        move |e| match e {
            nom::Err::Error(ve) | nom::Err::Failure(ve) => {
                let msg = pretty_verbose_error(label, ve);
                anyhow!(msg)
            }
            nom::Err::Incomplete(_) => anyhow!("incomplete input while parsing {}", label),
        }
    }

    fn pretty_verbose_error(label: &str, ve: VerboseError<&str>) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        let _ = writeln!(s, "parse error in {}:", label);
        for (frag, kind) in ve.errors {
            let show = frag
                .get(0..frag.find('\n').unwrap_or(frag.len()))
                .unwrap_or(frag);
            let _ = writeln!(s, "  at: {:?}  {:?}", show, kind);
        }
        s
    }

    /* ------------------------------- Utils ------------------------------- */

    fn range_from(base_len: usize, before: &str, after: &str) -> SourceRange {
        let start = base_len - before.len();
        let end = base_len - after.len();
        SourceRange { start, end }
    }

    fn flush_section_paragraph(
        node: &mut Heading,
        para_start: &mut Option<&str>,
        para_lines: &mut Vec<String>,
        current_rest: &str,
        base_len: usize,
    ) {
        if let Some(start) = *para_start {
            let text = para_lines.join("\n");
            let paragraph = Block::Paragraph(rt_text(&text));
            let range = range_from(base_len, start, current_rest);
            node.section
                .blocks
                .push(BlockWithSource::from_source(paragraph, range));
            para_lines.clear();
            *para_start = None;
        }
    }

    fn is_heading_line(s: &str) -> bool {
        // Heading when line starts with one-or-more '*' then at least one space.
        let mut chars = s.chars();
        let mut n = 0;
        while let Some('*') = chars.clone().next() {
            n += 1;
            chars.next();
        }
        n >= 1 && matches!(chars.next(), Some(' '))
    }

    fn count_stars(s: &str) -> usize {
        s.chars().take_while(|c| *c == '*').count()
    }

    fn till_eol(i: &str) -> PResult<'_, &str> {
        map(
            terminated(not_line_ending, opt(line_ending_ve)),
            |s: &str| s,
        )(i)
    }

    fn line_ending_ve(i: &str) -> PResult<'_, &str> {
        line_ending::<_, VerboseError<&str>>(i)
    }

    fn is_tag_char(c: char) -> bool {
        // conservative subset for tags; Org is more lenient.
        c.is_alphanumeric() || c == '_' || c == '-' || c == '@' || c == '+'
    }

    fn rt_text(s: &str) -> RichText {
        RichText {
            inlines: parse_inlines_str(s),
        }
    }

    /* --------------------------- INLINE MARKUP --------------------------- */

    fn parse_inlines_str(s: &str) -> Vec<Inline> {
        match parse_inlines(s) {
            Ok(("", mut v)) => {
                coalesce_text(&mut v);
                v
            }
            Ok((rest, mut v)) => {
                if !rest.is_empty() {
                    v.push(Inline::Text(rest.to_string()));
                }
                coalesce_text(&mut v);
                v
            }
            Err(_) => vec![Inline::Text(s.to_string())],
        }
    }

    fn parse_inlines(mut i: &str) -> PResult<'_, Vec<Inline>> {
        let mut out = Vec::new();
        while !i.is_empty() {
            match inline_atom(i) {
                Ok((r, node)) => {
                    out.push(node);
                    i = r;
                }
                Err(_) => {
                    let (r, ch) = anychar(i)?;
                    out.push(Inline::Text(ch.to_string()));
                    i = r;
                }
            }
        }
        Ok(("", out))
    }

    fn inline_atom(i: &str) -> PResult<'_, Inline> {
        alt((
            parse_link_bracketed,
            parse_target_inline,
            parse_footnote_ref,
            parse_code_like('~', |s| Inline::Code(s)),
            parse_code_like('=', |s| Inline::Verbatim(s)),
            parse_emph_with('*', Emphasis::Bold),
            parse_emph_with('/', Emphasis::Italic),
            parse_emph_with('_', Emphasis::Underline),
            parse_emph_with('+', Emphasis::Strike),
            parse_autolink,
            parse_entity_inline,
            parse_text_chunk,
        ))(i)
    }

    fn coalesce_text(xs: &mut Vec<Inline>) {
        let mut out = Vec::with_capacity(xs.len());
        for x in xs.drain(..) {
            if let (Some(Inline::Text(prev)), Inline::Text(s)) = (out.last_mut(), &x) {
                prev.push_str(s);
            } else {
                out.push(x);
            }
        }
        *xs = out;
    }

    fn parse_emph_with(delim: char, kind: Emphasis) -> impl Fn(&str) -> PResult<'_, Inline> {
        move |i: &str| {
            let (i, _) = char(delim)(i)?;
            if i.starts_with(' ') || i.starts_with('\n') {
                return Err(nom::Err::Error(VerboseError {
                    errors: vec![(i, VerboseErrorKind::Context("emphasis-open"))],
                }));
            }
            let (i, children) = parse_inlines_until(i, delim)?;
            let (i, _) = char(delim)(i)?;
            Ok((i, Inline::Emphasis { kind, children }))
        }
    }

    fn parse_inlines_until(mut i: &str, stop: char) -> PResult<'_, Vec<Inline>> {
        let mut out = Vec::new();
        loop {
            if i.is_empty() {
                return Err(nom::Err::Error(VerboseError {
                    errors: vec![(i, VerboseErrorKind::Context("unclosed-emphasis"))],
                }));
            }
            if i.starts_with(stop) {
                break;
            }
            match inline_atom(i) {
                Ok((r, node)) => {
                    out.push(node);
                    i = r;
                }
                Err(_) => {
                    let (r, ch) = anychar(i)?;
                    out.push(Inline::Text(ch.to_string()));
                    i = r;
                }
            }
        }
        Ok((i, out))
    }

    fn parse_code_like<F>(delim: char, make: F) -> impl Fn(&str) -> PResult<'_, Inline>
    where
        F: Fn(String) -> Inline + Copy,
    {
        move |i: &str| {
            let (i, _) = char(delim)(i)?;
            let (i, body) = take_till1(move |c| c == delim)(i)?;
            let (i, _) = char(delim)(i)?;
            Ok((i, make(body.to_string())))
        }
    }

    fn parse_link_bracketed(i: &str) -> PResult<'_, Inline> {
        let (i, _) = tag("[[")(i)?;
        if let Ok((i2, target)) = take_until::<&str, _, VerboseError<&str>>("][")(i) {
            let (i2, _) = tag("][")(i2)?;
            let (i2, desc_raw) = take_until::<&str, _, VerboseError<&str>>("]]")(i2)?;
            let (i2, _) = tag("]]")(i2)?;
            let kind = link_kind_from_target(target.trim());
            let desc = Some(parse_inlines_str(desc_raw));
            return Ok((i2, Inline::Link(Link { kind, desc })));
        }
        let (i, target) = take_until::<&str, _, VerboseError<&str>>("]]")(i)?;
        let (i, _) = tag("]]")(i)?;
        let kind = link_kind_from_target(target.trim());
        Ok((i, Inline::Link(Link { kind, desc: None })))
    }

    fn parse_autolink(i: &str) -> PResult<'_, Inline> {
        let (i, scheme) = alt((
            tag("https://"),
            tag("http://"),
            tag("mailto:"),
            tag("file:"),
            tag("id:"),
        ))(i)?;
        let (i, rest) =
            take_while1(|c: char| !c.is_whitespace() && c != ')' && c != ']' && c != '>')(i)?;
        let raw = format!("{}{}", scheme, rest);
        let kind = link_kind_from_target(&raw);
        Ok((i, Inline::Link(Link { kind, desc: None })))
    }

    fn link_kind_from_target(t: &str) -> LinkKind {
        let s = t.trim();
        if s.starts_with("http://") || s.starts_with("https://") {
            LinkKind::Http { url: s.to_string() }
        } else if let Some(rem) = s.strip_prefix("id:") {
            LinkKind::Id {
                id: rem.to_string(),
            }
        } else if let Some(rem) = s.strip_prefix("file:") {
            if let Some((path, search)) = rem.split_once("::") {
                LinkKind::File {
                    path: path.to_string(),
                    search: Some(search.to_string()),
                }
            } else {
                LinkKind::File {
                    path: rem.to_string(),
                    search: None,
                }
            }
        } else if s.contains(':') {
            let (proto, rest) = s.split_once(':').unwrap();
            LinkKind::Custom {
                protocol: proto.to_string(),
                target: rest.to_string(),
            }
        } else {
            LinkKind::File {
                path: s.to_string(),
                search: None,
            }
        }
    }

    fn parse_target_inline(i: &str) -> PResult<'_, Inline> {
        let (i, _) = tag("<<")(i)?;
        let (i, name) = take_until::<&str, _, VerboseError<&str>>(">>")(i)?;
        let (i, _) = tag(">>")(i)?;
        Ok((i, Inline::Target(name.to_string())))
    }

    fn parse_footnote_ref(i: &str) -> PResult<'_, Inline> {
        let (i, _) = tag("[fn:")(i)?;
        let (i, label) = take_until::<&str, _, VerboseError<&str>>("]")(i)?;
        let (i, _) = char(']')(i)?;
        Ok((i, Inline::FootnoteRef(label.to_string())))
    }

    fn parse_entity_inline(i: &str) -> PResult<'_, Inline> {
        let (i, _) = char('\\')(i)?;
        let (i, ident) = take_while1(|c: char| c.is_ascii_alphabetic())(i)?;
        Ok((i, Inline::Entity(format!("\\{}", ident))))
    }

    fn parse_text_chunk(i: &str) -> PResult<'_, Inline> {
        fn is_plain(c: char) -> bool {
            !matches!(
                c,
                '[' | '<' | '*' | '/' | '_' | '+' | '~' | '=' | '\\' | 'h' | 'f' | 'i' | 'm'
            )
        }
        let (i, s) = take_while1(is_plain)(i)?;
        Ok((i, Inline::Text(s.to_string())))
    }

    /* --------------------------- Preamble block -------------------------- */

    /// Parse file settings + preamble blocks until the first heading or EOF.
    fn parse_preamble(
        mut i: &str,
        base_len: usize,
    ) -> PResult<
        '_,
        (
            FileSettings,
            Option<String>,
            BTreeSet<Tag>,
            Vec<BlockWithSource>,
        ),
    > {
        let mut settings = FileSettings::default();
        let mut title: Option<String> = None;
        let mut file_tags: BTreeSet<Tag> = BTreeSet::new();
        let mut blocks: Vec<BlockWithSource> = Vec::new();
        let mut para_lines: Vec<String> = Vec::new();
        let mut para_start: Option<&str> = None;

        fn flush_paragraph(
            blocks: &mut Vec<BlockWithSource>,
            para_lines: &mut Vec<String>,
            para_start: &mut Option<&str>,
            current_rest: &str,
            base_len: usize,
        ) {
            if let Some(start) = *para_start {
                let paragraph = Block::Paragraph(rt_text(&para_lines.join("\n")));
                let range = range_from(base_len, start, current_rest);
                blocks.push(BlockWithSource::from_source(paragraph, range));
                para_lines.clear();
                *para_start = None;
            }
        }

        loop {
            let line_start = i;
            if i.is_empty() {
                break;
            }
            // Stop before the first heading.
            if is_heading_line(i) {
                break;
            }

            // Try known #+KEY: ...
            if let Ok((r, (key, val))) = parse_hash_key_value(i) {
                flush_paragraph(&mut blocks, &mut para_lines, &mut para_start, i, base_len);
                match key.to_ascii_lowercase().as_str() {
                    "title" => title = Some(val.trim().to_string()),
                    "filetags" => {
                        for t in parse_colon_tags_inline(val).into_iter() {
                            file_tags.insert(t);
                        }
                    }
                    "todo" | "todo_keywords" => {
                        if !val.trim().is_empty() {
                            let seq = TodoSequence {
                                items: val.split_whitespace().map(|s| s.to_string()).collect(),
                            };
                            settings.todo_sequences.push(seq);
                        }
                    }
                    // generic meta
                    other => {
                        settings.meta.insert(other.to_string(), val.to_string());
                    }
                }

                let range = range_from(base_len, line_start, r);
                blocks.push(BlockWithSource::from_source(
                    Block::Directive {
                        key: key.to_string(),
                        value: val.trim().to_string(),
                    },
                    range,
                ));
                i = r;
                continue;
            }

            // Otherwise treat as preamble content line.
            let (r, line) = till_eol(i)?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                flush_paragraph(&mut blocks, &mut para_lines, &mut para_start, r, base_len);
                let range = range_from(base_len, line_start, r);
                blocks.push(BlockWithSource::from_source(
                    Block::Paragraph(RichText::default()),
                    range,
                ));
            } else {
                if para_start.is_none() {
                    para_start = Some(line_start);
                }
                para_lines.push(line.to_string());
            }
            i = r;
        }

        flush_paragraph(&mut blocks, &mut para_lines, &mut para_start, i, base_len);

        Ok((i, (settings, title, file_tags, blocks)))
    }

    fn parse_hash_key_value(i: &str) -> PResult<'_, (&str, &str)> {
        // #+key: value
        map(
            tuple((
                tag("#+"),
                map(
                    take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_'),
                    |s: &str| s,
                ),
                tag(":"),
                space0,
                not_line_ending,
                opt(line_ending),
            )),
            |(_, key, _, _, val, _)| (key, val),
        )(i)
    }

    fn parse_colon_tags_inline(s: &str) -> Vec<Tag> {
        // expecting something like ":a:b:c:" or free text where we extract :x:
        let mut out = Vec::new();
        for part in s.split(':') {
            if part.is_empty() {
                continue;
            }
            if part.chars().all(is_tag_char) {
                out.push(Tag(part.to_string()));
            }
        }
        out
    }

    /* --------------------------- Headings section --------------------------- */

    /// Parse the entire heading tree (all top-level headings).
    fn parse_headings_tree<'a>(mut i: &'a str, base_len: usize) -> PResult<'a, Vec<Heading>> {
        let mut stack: Vec<Heading> = Vec::new(); // stack by levels (1-based)
        let mut roots: Vec<Heading> = Vec::new();

        while !i.is_empty() {
            if !is_heading_line(i) {
                // Skip blank or stray lines between nodes as paragraph into last node if any.
                let line_start = i;
                let (r, line) = till_eol(i)?;
                i = r;
                if let Some(last) = stack.last_mut() {
                    if !line.trim().is_empty() {
                        let range = range_from(base_len, line_start, i);
                        let paragraph = Block::Paragraph(rt_text(line));
                        last.section
                            .blocks
                            .push(BlockWithSource::from_source(paragraph, range));
                    }
                }
                continue;
            }

            // Parse a single headline line (no children yet).
            let (r, mut node) = parse_headline(i, base_len)?;
            let level = node.level;
            i = r;

            let mut para_lines: Vec<String> = Vec::new();
            let mut para_start: Option<&str> = None;

            // After headline, parse planning + drawers + section blocks until next heading or EOF,
            // but also collect potential *children* which are headings with greater level.
            loop {
                if i.is_empty() {
                    break;
                }
                // Child heading?
                if is_heading_line(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let next_level = count_stars(i) as u8;
                    if next_level > level {
                        // Parse child subtree(s) and attach.
                        let (r2, children) = parse_headings_at_level(i, next_level, base_len)?;
                        i = r2;
                        node.children.extend(children);
                        continue;
                    } else {
                        // sibling or higher-level; stop body parsing.
                        break;
                    }
                }

                // Planning lines (may be multiple).
                if let Ok((r2, (p, line_len, newline_len))) = parse_planning_line(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let start_offset = base_len - i.len();
                    let end_offset = start_offset + line_len + newline_len;
                    let range = SourceRange {
                        start: start_offset,
                        end: end_offset,
                    };
                    i = r2;
                    // Merge into node.planning (last one wins where both present).
                    if p.scheduled.is_some() {
                        node.planning.scheduled = p.scheduled;
                    }
                    if p.deadline.is_some() {
                        node.planning.deadline = p.deadline;
                    }
                    if p.closed.is_some() {
                        node.planning.closed = p.closed;
                    }
                    node.planning_range = match node.planning_range {
                        Some(existing) => Some(SourceRange {
                            start: existing.start,
                            end: range.end,
                        }),
                        None => Some(range),
                    };
                    continue;
                }

                // Drawers: PROPERTIES / LOGBOOK / generic drawer
                if let Ok((r2, pd)) = parse_properties_drawer(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.properties = pd;
                    node.properties_range = Some(range);
                    continue;
                }
                if let Ok((r2, (clock, rest_raw))) = parse_logbook_drawer(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.logbook.clock = clock;
                    node.logbook.raw = rest_raw;
                    node.logbook_range = Some(range);
                    continue;
                }
                if let Ok((r2, drawer)) = parse_generic_drawer(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.section
                        .blocks
                        .push(BlockWithSource::from_source(Block::Drawer(drawer), range));
                    continue;
                }

                // Horizontal rule
                if let Ok((r2, _)) = parse_hr(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.section
                        .blocks
                        .push(BlockWithSource::from_source(Block::HorizontalRule, range));
                    continue;
                }

                // Lists
                if let Ok((r2, list)) = parse_list(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.section
                        .blocks
                        .push(BlockWithSource::from_source(Block::List(list), range));
                    continue;
                }

                // Paragraph line
                let line_start = i;
                let (r2, line) = till_eol(i)?;
                let range = range_from(base_len, line_start, r2);
                if line.trim().is_empty() {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        line_start,
                        base_len,
                    );
                    i = r2;
                    node.section.blocks.push(BlockWithSource::from_source(
                        Block::Paragraph(RichText::default()),
                        range,
                    ));
                } else {
                    i = r2;
                    if para_start.is_none() {
                        para_start = Some(line_start);
                    }
                    para_lines.push(line.to_string());
                }
            }

            flush_section_paragraph(&mut node, &mut para_start, &mut para_lines, i, base_len);

            // Place node into the tree using the current stack.
            while let Some(top) = stack.last() {
                if top.level < level {
                    break;
                }
                let completed = stack.pop().unwrap();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(completed);
                } else {
                    roots.push(completed);
                }
            }
            stack.push(node);
        }

        // Drain remaining stack.
        while let Some(completed) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(completed);
            } else {
                roots.push(completed);
            }
        }

        Ok((i, roots))
    }

    /// Parse consecutive headings of a given `level` (used for child subtrees).
    fn parse_headings_at_level<'a>(
        mut i: &'a str,
        level: u8,
        base_len: usize,
    ) -> PResult<'a, Vec<Heading>> {
        let mut out = Vec::new();
        loop {
            if i.is_empty() || !is_heading_line(i) || count_stars(i) as u8 != level {
                break;
            }
            let (r, mut node) = parse_headline(i, base_len)?;
            debug_assert_eq!(node.level, level);
            i = r;

            let mut para_lines: Vec<String> = Vec::new();
            let mut para_start: Option<&str> = None;
            // body under this node, stopping at a sibling (same level) or ancestor (smaller level).
            loop {
                if i.is_empty() {
                    break;
                }
                if is_heading_line(i) {
                    let next = count_stars(i) as u8;
                    if next > level {
                        let (r2, kids) = parse_headings_at_level(i, next, base_len)?;
                        i = r2;
                        node.children.extend(kids);
                        continue;
                    }
                    if next <= level {
                        break;
                    }
                }

                if let Ok((r2, (p, line_len, newline_len))) = parse_planning_line(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let start_offset = base_len - i.len();
                    let end_offset = start_offset + line_len + newline_len;
                    let range = SourceRange {
                        start: start_offset,
                        end: end_offset,
                    };
                    i = r2;
                    if p.scheduled.is_some() {
                        node.planning.scheduled = p.scheduled;
                    }
                    if p.deadline.is_some() {
                        node.planning.deadline = p.deadline;
                    }
                    if p.closed.is_some() {
                        node.planning.closed = p.closed;
                    }
                    node.planning_range = match node.planning_range {
                        Some(existing) => Some(SourceRange {
                            start: existing.start,
                            end: range.end,
                        }),
                        None => Some(range),
                    };
                    continue;
                }
                if let Ok((r2, pd)) = parse_properties_drawer(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.properties = pd;
                    node.properties_range = Some(range);
                    continue;
                }
                if let Ok((r2, (clock, raw))) = parse_logbook_drawer(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.logbook.clock = clock;
                    node.logbook.raw = raw;
                    node.logbook_range = Some(range);
                    continue;
                }
                if let Ok((r2, drawer)) = parse_generic_drawer(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.section
                        .blocks
                        .push(BlockWithSource::from_source(Block::Drawer(drawer), range));
                    continue;
                }
                if let Ok((r2, _)) = parse_hr(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.section
                        .blocks
                        .push(BlockWithSource::from_source(Block::HorizontalRule, range));
                    continue;
                }
                if let Ok((r2, list)) = parse_list(i) {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        i,
                        base_len,
                    );
                    let range = range_from(base_len, i, r2);
                    i = r2;
                    node.section
                        .blocks
                        .push(BlockWithSource::from_source(Block::List(list), range));
                    continue;
                }

                let line_start = i;
                let (r2, line) = till_eol(i)?;
                let range = range_from(base_len, line_start, r2);
                if line.trim().is_empty() {
                    flush_section_paragraph(
                        &mut node,
                        &mut para_start,
                        &mut para_lines,
                        line_start,
                        base_len,
                    );
                    i = r2;
                    node.section.blocks.push(BlockWithSource::from_source(
                        Block::Paragraph(RichText::default()),
                        range,
                    ));
                } else {
                    i = r2;
                    if para_start.is_none() {
                        para_start = Some(line_start);
                    }
                    para_lines.push(line.to_string());
                }
            }

            flush_section_paragraph(&mut node, &mut para_start, &mut para_lines, i, base_len);

            out.push(node);
        }
        Ok((i, out))
    }

    /// Parse a single headline line (no trailing body).
    fn parse_headline(i: &str, base_len: usize) -> PResult<'_, Heading> {
        let start = i;
        let (i, stars) = recognize(many1(char('*')))(i)?;
        let level = stars.len() as u8;
        let (i, _) = space1(i)?;

        let (i, todo_opt) = opt(terminated(
            map(take_while1(|c: char| c.is_ascii_uppercase()), |s: &str| {
                s.to_string()
            }),
            space1,
        ))(i)?;

        let (i, prio_opt) = opt(delimited(tag("[#"), map(anychar, |c| c), tag("]")))(i)?;
        let (i, _) = if prio_opt.is_some() {
            space0(i)?
        } else {
            (i, "")
        };

        let (i, title_text) = map(recognize(many0(is_not("\n"))), |s: &str| s.trim_end())(i)?;

        let mut tags = BTreeSet::<Tag>::new();
        let mut title_str = title_text;
        if let Some(pos) = title_text.rfind(" :") {
            let trail = &title_text[pos + 1..];
            if trail.starts_with(':') && trail.ends_with(':') {
                let mut cur = trail.trim();
                cur = cur.trim_end_matches(':');
                for t in cur.split(':').filter(|s| !s.is_empty()) {
                    if t.chars().all(is_tag_char) {
                        tags.insert(Tag(t.to_string()));
                    }
                }
                title_str = &title_text[..pos].trim_end();
            }
        }

        let (i, _) = opt(line_ending_ve)(i)?;

        let mut h = Heading::new(
            level,
            RichText {
                inlines: parse_inlines_str(title_str),
            },
        );
        if let Some(todo) = todo_opt {
            h.todo = Some(TodoKeyword {
                text: todo,
                is_done: false,
            });
        }
        if let Some(p) = prio_opt {
            h.priority = Some(Priority(p));
        }
        h.tags = tags;

        h.headline_range = Some(range_from(base_len, start, i));

        Ok((i, h))
    }

    /* --------------------------- Planning & Drawers --------------------------- */

    fn parse_planning_line(i: &str) -> PResult<'_, (Planning, usize, usize)> {
        let input = i;
        // e.g.: SCHEDULED: <2025-11-15 12:00> DEADLINE: <...>  CLOSED: [2025-11-15 14:10]
        let (rest_after_line, line) = till_eol(i)?;
        let mut rest = line;
        let mut matched = false;
        let mut p = Planning::default();

        while !rest.trim().is_empty() {
            // try each field
            if let Ok((r, ts)) = preceded_ws(tag("SCHEDULED:"), parse_timestamp)(rest) {
                p.scheduled = Some(ts);
                rest = r;
                matched = true;
                continue;
            }
            if let Ok((r, ts)) = preceded_ws(tag("DEADLINE:"), parse_timestamp)(rest) {
                p.deadline = Some(ts);
                rest = r;
                matched = true;
                continue;
            }
            if let Ok((r, ts)) = preceded_ws(tag("CLOSED:"), parse_timestamp)(rest) {
                p.closed = Some(ts);
                rest = r;
                matched = true;
                continue;
            }
            // nothing matched -> not a planning line
            return Err(nom::Err::Error(VerboseError {
                errors: vec![(i, VerboseErrorKind::Context("planning"))],
            }));
        }

        if !matched {
            return Err(nom::Err::Error(VerboseError {
                errors: vec![(i, VerboseErrorKind::Context("planning-empty"))],
            }));
        }

        let consumed = input.len() - rest_after_line.len();
        let line_len = line.len();
        let newline_len = consumed.saturating_sub(line_len);

        Ok((rest_after_line, (p, line_len, newline_len)))
    }

    fn preceded_ws<'a, F, O>(
        prefix: F,
        inner: impl Fn(&'a str) -> PResult<'a, O>,
    ) -> impl Fn(&'a str) -> PResult<'a, O>
    where
        F: Fn(&'a str) -> PResult<'a, &'a str>,
    {
        move |i: &'a str| {
            let (i, _) = space0(i)?;
            let (i, _) = prefix(i)?;
            let (i, _) = space0(i)?;
            inner(i)
        }
    }

    fn parse_properties_drawer(i: &str) -> PResult<'_, PropertyDrawer> {
        // :PROPERTIES:\n :KEY: value\n ... \n:END:
        let (i, _) = terminated(tag(":PROPERTIES:"), line_ending)(i)?;
        let mut props = indexmap::IndexMap::<String, String>::new();
        let mut rest = i;
        loop {
            if let Ok((r, _)) = terminated(tag(":END:"), opt(line_ending_ve))(rest) {
                return Ok((r, PropertyDrawer { props }));
            }
            let (r, (k, v)) = parse_property_line(rest)?;
            props.insert(k.to_string(), v.to_string());
            rest = r;
        }
    }

    fn parse_property_line(i: &str) -> PResult<'_, (&str, &str)> {
        //  :KEY: value
        map(
            tuple((
                space0,
                char(':'),
                take_while1(|c: char| c.is_ascii_uppercase() || c == '_' || c == '-'),
                char(':'),
                space0,
                not_line_ending,
                opt(line_ending_ve),
            )),
            |(_, _, key, _, _, val, _)| (key, val),
        )(i)
    }

    fn parse_logbook_drawer(i: &str) -> PResult<'_, (Vec<ClockEntry>, Vec<String>)> {
        // :LOGBOOK:\n CLOCK: [..]--[..] => 1:23\n ... \n:END:
        let (i, _) = terminated(tag(":LOGBOOK:"), line_ending)(i)?;
        let mut clocks = Vec::new();
        let mut raw = Vec::new();
        let mut rest = i;
        loop {
            if let Ok((r, _)) = terminated(tag(":END:"), opt(line_ending_ve))(rest) {
                return Ok((r, (clocks, raw)));
            }
            if let Ok((r, ce)) = parse_clock_line(rest) {
                clocks.push(ce);
                rest = r;
                continue;
            }
            let (r, line) = till_eol(rest)?;
            raw.push(line.to_string());
            rest = r;
        }
    }

    fn parse_clock_line(i: &str) -> PResult<'_, ClockEntry> {
        // CLOCK: [2025-11-15 10:00]--[2025-11-15 11:30] => 1:30
        let (i, _) = space0(i)?;
        let (i, _) = tag("CLOCK:")(i)?;
        let (i, _) = space1(i)?;
        let (i, start) = parse_timestamp(i)?;
        let (i, _) = space0(i)?;
        let (i, _) = tag("--")(i)?;
        let (i, _) = space0(i)?;
        let (i, end) = opt(parse_timestamp)(i)?;
        let (i, minutes) = opt(parse_clock_minutes)(i)?;
        let (i, _) = opt(line_ending_ve)(i)?;

        Ok((
            i,
            ClockEntry {
                start,
                end,
                minutes,
                raw: None,
            },
        ))
    }

    fn parse_clock_minutes(i: &str) -> PResult<'_, i64> {
        // " => H:MM" or " => M:SS" — we’ll parse as hours:minutes to minutes
        let (i, _) = space0(i)?;
        let (i, _) = tag("=>")(i)?;
        let (i, _) = space0(i)?;
        let (i, hours) = map_res(digit1, |s: &str| s.parse::<i64>())(i)?;
        let (i, _) = char(':')(i)?;
        let (i, mins) = map_res(digit1, |s: &str| s.parse::<i64>())(i)?;
        Ok((i, hours * 60 + mins))
    }

    fn parse_generic_drawer(i: &str) -> PResult<'_, Drawer> {
        // :NAME:\n ... \n:END:
        let (i, name) = terminated(
            delimited(
                char(':'),
                take_while1(|c: char| c.is_ascii_uppercase()),
                char(':'),
            ),
            line_ending,
        )(i)?;
        if name == "PROPERTIES" || name == "LOGBOOK" {
            return Err(nom::Err::Error(VerboseError {
                errors: vec![(i, VerboseErrorKind::Context("drawer"))],
            }));
        }
        let mut content_lines = Vec::new();
        let mut rest = i;
        loop {
            if let Ok((r, _)) = terminated(tag(":END:"), opt(line_ending_ve))(rest) {
                let blocks = parse_blocks_from_lines(&content_lines);
                return Ok((
                    r,
                    Drawer {
                        name: name.to_string(),
                        content: blocks,
                    },
                ));
            }
            let (r, line) = till_eol(rest)?;
            content_lines.push(line);
            rest = r;
        }
    }

    /* ----------------------------- Blocks/Lists ----------------------------- */

    fn parse_hr(i: &str) -> PResult<'_, ()> {
        // 5+ dashes alone on a line
        map(
            terminated(tuple((space0, many1(char('-')), space0)), line_ending),
            |_| (),
        )(i)
    }

    fn parse_list(mut i: &str) -> PResult<'_, List> {
        // Simple contiguous list (unordered '-' or '+' or ordered '1.' style).
        // We read at least one item and stop when a non-list line appears.
        let (i0, (kind, first)) = parse_list_item(i)?;
        let mut items = vec![first];
        let list_kind = kind;
        i = i0;

        loop {
            let try_next = parse_list_item(i);
            match try_next {
                Ok((r, (k, it))) if k == list_kind => {
                    items.push(it);
                    i = r;
                }
                _ => break,
            }
        }

        Ok((
            i,
            List {
                kind: list_kind,
                items,
            },
        ))
    }

    fn parse_list_item(i: &str) -> PResult<'_, (ListKind, ListItem)> {
        // "- [ ] text", "+ text", "1. text"
        // label (term) for description lists is out of scope here.
        let unordered = map(tuple((space0, alt((char('-'), char('+'))), space1)), |_| {
            ListKind::Unordered
        });
        let ordered = map(
            tuple((space0, digit1, alt((char('.'), char(')'))), space1)),
            |_| ListKind::Ordered,
        );
        let (i, kind) = alt((unordered, ordered))(i)?;
        let (i, checkbox) = opt(parse_checkbox)(i)?;
        let (i, text) = till_eol(i)?;

        let item = ListItem {
            label: None,
            content: vec![Block::Paragraph(RichText {
                inlines: parse_inlines_str(text.trim_end()),
            })],
            checkbox,
            counter: None,
            tags: BTreeSet::new(),
        };
        Ok((i, (kind, item)))
    }

    fn parse_checkbox(i: &str) -> PResult<'_, Checkbox> {
        let (i, _) = char('[')(i)?;
        let (i, state) = alt((
            map(char(' '), |_| Checkbox::Empty),
            map(char('-'), |_| Checkbox::Partial),
            map(char('X'), |_| Checkbox::Checked),
            map(char('x'), |_| Checkbox::Checked),
        ))(i)?;
        let (i, _) = char(']')(i)?;
        let (i, _) = space1(i)?;
        Ok((i, state))
    }

    fn parse_blocks_from_lines(lines: &[&str]) -> Vec<Block> {
        // Minimal: join paragraphs separated by blank lines; parse lists per-line later if needed.
        let mut blocks = Vec::new();
        let mut para = Vec::<String>::new();

        let flush_para = |para: &mut Vec<String>, blocks: &mut Vec<Block>| {
            if !para.is_empty() {
                let text = para.join("\n");
                blocks.push(Block::Paragraph(RichText {
                    inlines: parse_inlines_str(&text),
                }));
                para.clear();
            }
        };

        for &line in lines {
            if line.trim().is_empty() {
                flush_para(&mut para, &mut blocks);
            } else {
                para.push(line.to_string());
            }
        }
        flush_para(&mut para, &mut blocks);
        blocks
    }

    #[cfg(test)]
    mod tests {
        use super::{parse_headline, parse_inlines_str};
        use crate::core::{Inline, Link, LinkKind};

        #[test]
        fn emphasis_nested() {
            let v = parse_inlines_str("This is *bold and /italic/* text* end.");
            assert!(v.iter().any(|i| matches!(i, Inline::Emphasis { .. })));
            assert!(
                v.iter()
                    .any(|i| matches!(i, Inline::Text(t) if t.contains("This is ")))
            );
        }

        #[test]
        fn code_and_verbatim() {
            let v = parse_inlines_str("Use ~println!()~ with =NO_EXPAND=.");
            assert!(matches!(v[1], Inline::Code(_)));
            assert!(matches!(v[3], Inline::Verbatim(_)));
        }

        #[test]
        fn links_and_autolinks() {
            let v1 = parse_inlines_str("See [[https://example.com][site]]!");
            match &v1[1] {
                Inline::Link(Link {
                    kind: LinkKind::Http { url },
                    desc: Some(desc),
                }) => {
                    assert!(url.starts_with("https://"));
                    assert!(!desc.is_empty());
                }
                other => panic!("expected bracketed link, got {:?}", other),
            }

            let v2 = parse_inlines_str("Visit https://example.com now.");
            match &v2[1] {
                Inline::Link(Link {
                    kind: LinkKind::Http { url },
                    desc: None,
                }) => {
                    assert!(url.starts_with("https://"));
                }
                other => panic!("expected autolink, got {:?}", other),
            }
        }

        #[test]
        fn targets_and_footnotes() {
            let v = parse_inlines_str("Jump to <<here>> and see [fn:1].");
            assert!(v.iter().any(|i| matches!(i, Inline::Target(_))));
            assert!(v.iter().any(|i| matches!(i, Inline::FootnoteRef(_))));
        }

        #[test]
        fn headline_with_markup_and_tags() {
            let text = "* TODO Title with *bold* and [[id:abc][ref]] :tag:\n";
            let (_, h) = parse_headline(text, text.len()).unwrap();
            assert_eq!(h.level, 1);
            assert!(h.tags.iter().any(|t| t.0 == "tag"));
            assert!(
                h.title
                    .inlines
                    .iter()
                    .any(|i| matches!(i, Inline::Emphasis { .. }))
            );
            assert!(h.title.inlines.iter().any(|i| matches!(i, Inline::Link(_))));
        }
    }

    /* ----------------------------- Timestamps ----------------------------- */

    fn parse_timestamp(i: &str) -> PResult<'_, Timestamp> {
        // Active: <YYYY-MM-DD [HH:MM]>
        // Inactive: [YYYY-MM-DD [HH:MM]]
        let active = i.starts_with('<');
        let (i, (open, date, time_opt, _day_opt, close)) = tuple((
            alt((char('<'), char('['))),
            parse_date,
            opt(preceded(space1, parse_time)),
            opt(preceded(space1, take_while1(|c: char| c.is_alphabetic()))), // Day of week; ignored
            alt((char('>'), char(']'))),
        ))(i)?;

        let _ = (open, close); // just to silence warnings; we rely on brackets for active state
        let ts = Timestamp {
            active,
            date,
            time: time_opt,
            tz: None,
            end: None,
            repeater: None,
            delay: None,
        };
        Ok((i, ts))
    }

    fn parse_date(i: &str) -> PResult<'_, NaiveDate> {
        map_res(
            tuple((
                map_res(take_while_m_n(4, 4, char_is_digit), |s: &str| {
                    s.parse::<i32>()
                }),
                char('-'),
                map_res(take_while_m_n(2, 2, char_is_digit), |s: &str| {
                    s.parse::<u32>()
                }),
                char('-'),
                map_res(take_while_m_n(2, 2, char_is_digit), |s: &str| {
                    s.parse::<u32>()
                }),
            )),
            |(y, _, m, _, d)| NaiveDate::from_ymd_opt(y, m, d).ok_or_else(|| "invalid date"),
        )(i)
    }

    fn parse_time(i: &str) -> PResult<'_, NaiveTime> {
        map_res(
            tuple((
                map_res(take_while_m_n(1, 2, char_is_digit), |s: &str| {
                    s.parse::<u32>()
                }),
                char(':'),
                map_res(take_while_m_n(2, 2, char_is_digit), |s: &str| {
                    s.parse::<u32>()
                }),
            )),
            |(h, _, m)| NaiveTime::from_hms_opt(h, m, 0).ok_or_else(|| "invalid time"),
        )(i)
    }

    fn take_while_m_n<F>(m: usize, n: usize, cond: F) -> impl Fn(&str) -> PResult<'_, &str>
    where
        F: Fn(char) -> bool + Copy,
    {
        move |i: &str| {
            let (i, out) = take_while(cond)(i)?;
            if out.len() < m || out.len() > n {
                Err(nom::Err::Error(VerboseError {
                    errors: vec![(i, VerboseErrorKind::Context("m_n"))],
                }))
            } else {
                Ok((i, out))
            }
        }
    }

    fn char_is_digit(c: char) -> bool {
        c.is_ascii_digit()
    }
}

pub mod format {
    use super::core::*;

    pub fn format_org_file(file: &OrgFile) -> String {
        let source = file.source_text.as_deref();
        let mut out = String::new();

        for block in &file.preamble {
            append_block(&mut out, block, source);
        }

        for heading in &file.headings {
            format_heading(&mut out, heading, source, true);
        }

        out
    }

    fn append_block(out: &mut String, block: &BlockWithSource, source: Option<&str>) {
        if let (Some(range), Some(src)) = (block.source, source) {
            out.push_str(range.slice(src));
            return;
        }

        out.push_str(&render_block(&block.block));
    }

    fn render_block(block: &Block) -> String {
        match block {
            Block::Paragraph(text) => {
                let mut buf = render_rich_text(&text.inlines);
                buf.push('\n');
                buf
            }
            Block::List(list) => render_list(list),
            Block::Quote(blocks) => {
                let mut buf = String::new();
                for blk in blocks {
                    for line in render_block(blk).lines() {
                        buf.push_str("> ");
                        buf.push_str(line);
                        buf.push('\n');
                    }
                }
                buf
            }
            Block::Example { raw } => {
                let mut buf = String::new();
                buf.push_str("#+BEGIN_EXAMPLE\n");
                buf.push_str(raw);
                if !raw.ends_with('\n') {
                    buf.push('\n');
                }
                buf.push_str("#+END_EXAMPLE\n");
                buf
            }
            Block::SrcBlock(src) => {
                let mut buf = String::new();
                buf.push_str("#+BEGIN_SRC");
                if let Some(lang) = &src.language {
                    buf.push(' ');
                    buf.push_str(lang);
                }
                if !src.parameters.is_empty() {
                    for (k, v) in &src.parameters {
                        buf.push(' ');
                        buf.push_str(k);
                        buf.push_str("=");
                        buf.push_str(v);
                    }
                }
                buf.push('\n');
                buf.push_str(&src.code);
                if !src.code.ends_with('\n') {
                    buf.push('\n');
                }
                buf.push_str("#+END_SRC\n");
                buf
            }
            Block::Drawer(drawer) => {
                let mut buf = String::new();
                buf.push(':');
                buf.push_str(&drawer.name);
                buf.push_str(":\n");
                for blk in &drawer.content {
                    buf.push_str(&render_block(blk));
                }
                buf.push_str(":END:\n");
                buf
            }
            Block::Table(table) => {
                let mut buf = String::new();
                for line in &table.raw {
                    buf.push_str(line);
                    if !line.ends_with('\n') {
                        buf.push('\n');
                    }
                }
                buf
            }
            Block::HorizontalRule => "-----\n".to_string(),
            Block::Comment(text) => {
                let mut buf = String::new();
                buf.push_str(text);
                buf.push('\n');
                buf
            }
            Block::Directive { key, value } => {
                let mut buf = String::new();
                buf.push_str("#+");
                buf.push_str(key);
                buf.push_str(": ");
                buf.push_str(value);
                buf.push('\n');
                buf
            }
            Block::Unknown { raw, .. } => {
                let mut buf = raw.clone();
                if !raw.ends_with('\n') {
                    buf.push('\n');
                }
                buf
            }
        }
    }

    fn render_list(list: &List) -> String {
        let mut buf = String::new();
        for item in &list.items {
            let prefix = match list.kind {
                ListKind::Unordered => "-",
                ListKind::Ordered => "1.",
                ListKind::Description => "::",
            };
            buf.push_str(prefix);
            buf.push(' ');

            if let Some(cb) = item.checkbox {
                let symbol = match cb {
                    Checkbox::Empty => ' ',
                    Checkbox::Partial => '-',
                    Checkbox::Checked => 'X',
                };
                buf.push('[');
                buf.push(symbol);
                buf.push_str("] ");
            }

            if let Some(label) = &item.label {
                buf.push_str(&render_rich_text(&label.inlines));
                buf.push_str(" :: ");
            }

            if item.content.is_empty() {
                buf.push('\n');
            } else {
                // Render first block inline when possible.
                let mut first = true;
                for blk in &item.content {
                    let rendered = render_block(blk);
                    if first {
                        buf.push_str(rendered.trim_end_matches('\n'));
                        buf.push('\n');
                        first = false;
                    } else {
                        buf.push_str("  ");
                        buf.push_str(&rendered);
                    }
                }
            }
        }
        buf
    }

    fn format_heading(
        out: &mut String,
        heading: &Heading,
        source: Option<&str>,
        is_root_level: bool,
    ) {
        if !is_root_level && !out.ends_with('\n') {
            out.push('\n');
        }

        if let (Some(range), Some(src)) = (heading.headline_range, source) {
            out.push_str(range.slice(src));
        } else {
            out.push_str(&render_headline(heading));
        }

        if let Some(range) = heading.planning_range {
            if let Some(src) = source {
                out.push_str(range.slice(src));
            }
        } else if heading.planning.scheduled.is_some()
            || heading.planning.deadline.is_some()
            || heading.planning.closed.is_some()
        {
            out.push_str(&render_planning(&heading.planning));
        }

        if let Some(range) = heading.properties_range {
            if let Some(src) = source {
                out.push_str(range.slice(src));
            }
        } else if !heading.properties.props.is_empty() {
            out.push_str(&render_properties(&heading.properties));
        }

        if let Some(range) = heading.logbook_range {
            if let Some(src) = source {
                out.push_str(range.slice(src));
            }
        } else if !heading.logbook.clock.is_empty() || !heading.logbook.raw.is_empty() {
            out.push_str(&render_logbook(&heading.logbook));
        }

        for block in &heading.section.blocks {
            append_block(out, block, source);
        }

        for child in &heading.children {
            format_heading(out, child, source, false);
        }
    }

    fn render_headline(heading: &Heading) -> String {
        let mut buf = String::new();
        buf.push_str(&"*".repeat(heading.level as usize));
        buf.push(' ');

        if let Some(todo) = &heading.todo {
            buf.push_str(&todo.text);
            buf.push(' ');
        }

        if let Some(priority) = &heading.priority {
            buf.push_str(&format!("[#{}] ", priority.0));
        }

        buf.push_str(&render_rich_text(&heading.title.inlines));

        if !heading.tags.is_empty() {
            buf.push(' ');
            buf.push(':');
            for tag in &heading.tags {
                buf.push_str(&tag.0);
                buf.push(':');
            }
        }
        buf.push('\n');
        buf
    }

    fn render_planning(plan: &Planning) -> String {
        let mut parts = Vec::new();
        if let Some(ts) = &plan.scheduled {
            parts.push(format!("SCHEDULED: {}", render_timestamp(ts)));
        }
        if let Some(ts) = &plan.deadline {
            parts.push(format!("DEADLINE: {}", render_timestamp(ts)));
        }
        if let Some(ts) = &plan.closed {
            parts.push(format!("CLOSED: {}", render_timestamp(ts)));
        }
        let mut line = parts.join(" ");
        line.push('\n');
        line
    }

    fn render_properties(props: &PropertyDrawer) -> String {
        let mut buf = String::new();
        buf.push_str(":PROPERTIES:\n");
        for (k, v) in &props.props {
            buf.push(':');
            buf.push_str(k);
            buf.push_str(": ");
            buf.push_str(v);
            buf.push('\n');
        }
        buf.push_str(":END:\n");
        buf
    }

    fn render_logbook(log: &Logbook) -> String {
        let mut buf = String::new();
        buf.push_str(":LOGBOOK:\n");
        for clock in &log.clock {
            buf.push_str("CLOCK: ");
            buf.push_str(&render_timestamp(&clock.start));
            if let Some(end) = &clock.end {
                buf.push_str("--");
                buf.push_str(&render_timestamp(end));
            }
            if let Some(mins) = clock.minutes {
                let hours = mins / 60;
                let minutes = mins % 60;
                buf.push_str(&format!(" => {}:{:02}", hours, minutes));
            }
            buf.push('\n');
        }
        for raw in &log.raw {
            buf.push_str(raw);
            buf.push('\n');
        }
        buf.push_str(":END:\n");
        buf
    }

    fn render_timestamp(ts: &Timestamp) -> String {
        let mut buf = String::new();
        buf.push(if ts.active { '<' } else { '[' });
        buf.push_str(&ts.date.format("%Y-%m-%d").to_string());
        if let Some(time) = ts.time {
            buf.push(' ');
            buf.push_str(&time.format("%H:%M").to_string());
        }
        if let Some(Repeater { kind, interval }) = &ts.repeater {
            buf.push(' ');
            let sym = match kind {
                RepeaterKind::FromLast => "+",
                RepeaterKind::FromBase => "++",
                RepeaterKind::FromNow => ".+",
            };
            buf.push_str(sym);
            buf.push_str(&render_offset(interval));
        }
        if let Some(delay) = &ts.delay {
            buf.push(' ');
            buf.push(if delay.before { '-' } else { '+' });
            buf.push_str(&render_offset(&delay.offset));
        }
        buf.push(if ts.active { '>' } else { ']' });
        buf
    }

    fn render_offset(offset: &DateOffset) -> String {
        if offset.weeks != 0 {
            format!("{}w", offset.weeks.abs())
        } else if offset.days != 0 {
            format!("{}d", offset.days.abs())
        } else if offset.months != 0 {
            format!("{}m", offset.months.abs())
        } else if offset.years != 0 {
            format!("{}y", offset.years.abs())
        } else if offset.hours != 0 {
            format!("{}h", offset.hours.abs())
        } else {
            format!("{}m", offset.minutes.abs())
        }
    }

    fn render_rich_text(inlines: &[Inline]) -> String {
        let mut buf = String::new();
        for inline in inlines {
            match inline {
                Inline::Text(t) => buf.push_str(t),
                Inline::Emphasis { kind, children } => {
                    let marker = match kind {
                        Emphasis::Bold => '*',
                        Emphasis::Italic => '/',
                        Emphasis::Underline => '_',
                        Emphasis::Strike => '+',
                        Emphasis::Mark => '=',
                    };
                    buf.push(marker);
                    buf.push_str(&render_rich_text(children));
                    buf.push(marker);
                }
                Inline::Code(code) => {
                    buf.push('~');
                    buf.push_str(code);
                    buf.push('~');
                }
                Inline::Verbatim(verbatim) => {
                    buf.push('=');
                    buf.push_str(verbatim);
                    buf.push('=');
                }
                Inline::Link(link) => {
                    buf.push_str("[[");
                    buf.push_str(&render_link_target(&link.kind));
                    if let Some(desc) = &link.desc {
                        buf.push_str("][");
                        buf.push_str(&render_rich_text(desc));
                    }
                    buf.push_str("]]");
                }
                Inline::Target(target) => {
                    buf.push_str("<<");
                    buf.push_str(target);
                    buf.push_str(">>");
                }
                Inline::FootnoteRef(label) => {
                    buf.push_str("[fn:");
                    buf.push_str(label);
                    buf.push(']');
                }
                Inline::Entity(entity) => buf.push_str(entity),
                Inline::Unknown { raw, .. } => buf.push_str(raw),
            }
        }
        buf
    }

    fn render_link_target(kind: &LinkKind) -> String {
        match kind {
            LinkKind::File { path, search } => {
                if let Some(search) = search {
                    format!("file:{}::{}", path, search)
                } else {
                    format!("file:{}", path)
                }
            }
            LinkKind::Http { url } => url.clone(),
            LinkKind::Id { id } => format!("id:{}", id),
            LinkKind::Custom { protocol, target } => format!("{}:{}", protocol, target),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::core::{Block, Inline, RichText};
        use crate::parser::parse_org_from_str;

        #[test]
        fn formatter_round_trips_original_text() {
            let input = r#"#+title: Demo
#+filetags: :foo:

* TODO Task :tag:
SCHEDULED: <2025-11-15>
Paragraph line one
Paragraph line two

** DONE Child
Child text
"#;

            let file = parse_org_from_str(None, input).expect("parse");
            let formatted = format_org_file(&file);
            assert_eq!(formatted, input);
        }

        #[test]
        fn formatter_preserves_context_when_inserting_block() {
            let input = r#"* TODO Task
Paragraph line one
Paragraph line two
"#;
            let mut file = parse_org_from_str(None, input).expect("parse");
            let heading = file.headings.get_mut(0).expect("heading");
            heading.section.blocks.insert(
                0,
                BlockWithSource::new(Block::Paragraph(RichText {
                    inlines: vec![Inline::Text("Inserted note".into())],
                })),
            );
            let expected = r#"* TODO Task
Inserted note
Paragraph line one
Paragraph line two
"#;
            let formatted = format_org_file(&file);
            assert_eq!(formatted, expected);
        }
    }
}

pub mod projectors {
    pub mod agenda_projector {
        use crate::agenda::{AgendaItem, AgendaWhenKind};
        use crate::core::*;
        use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

        #[derive(Debug, Clone, Copy, Default)]
        pub struct ProjectOptions {
            pub include_todos: bool,
        }

        /// Project agenda items from a single file.
        pub fn project_file(file: &OrgFile) -> Vec<AgendaItem> {
            project_file_with_options(file, ProjectOptions::default())
        }

        /// Project agenda items from many files.
        pub fn project_files<'a>(files: impl IntoIterator<Item = &'a OrgFile>) -> Vec<AgendaItem> {
            project_files_with_options(files, ProjectOptions::default())
        }

        /// Project agenda items from a single file with options.
        pub fn project_file_with_options(file: &OrgFile, opts: ProjectOptions) -> Vec<AgendaItem> {
            let mut out = Vec::new();
            let mut context = Vec::<String>::new();
            for h in &file.headings {
                walk_heading(file, h, &mut context, &mut out, opts);
            }
            out
        }

        /// Project agenda items from many files with options.
        pub fn project_files_with_options<'a>(
            files: impl IntoIterator<Item = &'a OrgFile>,
            opts: ProjectOptions,
        ) -> Vec<AgendaItem> {
            let mut all = Vec::new();
            for f in files {
                all.extend(project_file_with_options(f, opts));
            }
            all
        }

        fn walk_heading(
            file: &OrgFile,
            h: &Heading,
            path: &mut Vec<String>,
            out: &mut Vec<AgendaItem>,
            opts: ProjectOptions,
        ) {
            path.push(h.title.plain_text());

            let mut has_planning = false;

            // SCHEDULED
            if let Some(ts) = &h.planning.scheduled {
                has_planning = true;
                out.push(make_item(file, h, AgendaWhenKind::Scheduled, ts, &path));
            }

            // DEADLINE
            if let Some(ts) = &h.planning.deadline {
                has_planning = true;
                out.push(make_item(file, h, AgendaWhenKind::Deadline, ts, &path));
            }

            // CLOSED
            if let Some(ts) = &h.planning.closed {
                has_planning = true;
                out.push(make_item(file, h, AgendaWhenKind::Closed, ts, &path));
            }

            if opts.include_todos {
                if let Some(todo) = &h.todo {
                    if !todo.is_done && !has_planning {
                        out.push(AgendaItem::new(
                            file.id,
                            h.id,
                            AgendaWhenKind::Todo,
                            todo_placeholder_span(),
                            false,
                            h.title.plain_text(),
                            Some(todo.clone()),
                            h.priority,
                            h.tags.iter().cloned().collect(),
                            path.clone(),
                        ));
                    }
                }
            }

            for c in &h.children {
                walk_heading(file, c, path, out, opts);
            }
            path.pop();
        }

        fn make_item(
            file: &OrgFile,
            h: &Heading,
            kind: AgendaWhenKind,
            ts: &Timestamp,
            ctx: &[String],
        ) -> AgendaItem {
            AgendaItem::new(
                file.id,
                h.id,
                kind,
                ts_to_span(ts),
                ts.active,
                h.title.plain_text(),
                h.todo.clone(),
                h.priority,
                h.tags.iter().cloned().collect(),
                ctx.to_vec(),
            )
        }

        fn ts_to_span(ts: &Timestamp) -> TimeSpan {
            let start_time: NaiveTime = ts
                .time
                .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            let start = NaiveDateTime::new(ts.date, start_time);

            let end = ts.end.as_ref().map(|e| {
                let end_date = e.date.unwrap_or(ts.date);
                let end_time = e.time.unwrap_or(start_time);
                NaiveDateTime::new(end_date, end_time)
            });

            TimeSpan { start, end }
        }

        fn todo_placeholder_span() -> TimeSpan {
            let start = NaiveDate::MIN
                .and_hms_opt(0, 0, 0)
                .expect("valid minimum datetime");
            TimeSpan { start, end: None }
        }
    }

    pub mod journal_new_entry_projector {
        use crate::core::*;
        use crate::format::format_org_file;
        use crate::parse_org_from_str;
        use crate::workspace::{OrgWorkspace, RelPath};
        use chrono::{Duration, NaiveDate, NaiveTime};
        use indexmap::IndexMap;
        use std::collections::BTreeSet;
        use uuid::Uuid;

        /* --------------------------- Reschedule policy --------------------------- */

        /// How to adjust timestamps when carrying tasks forward.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum RescheduleRule {
            /// Don’t touch these timestamps.
            NoChange,
            /// Always set (date/time according to policy) to the new entry date.
            SetToTarget,
            /// Set only if the original date is before the new entry date (overdue).
            ToTargetIfOverdue,
            /// Shift by (target_date - shift_from) days; if `shift_from` is None, this is a no-op.
            ShiftByDeltaDays,
        }

        /// Policy controlling how SCHEDULED/DEADLINE are rewritten.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct ReschedulePolicy {
            pub scheduled_rule: RescheduleRule,
            pub deadline_rule: RescheduleRule,
            /// Keep the original time-of-day if present.
            pub keep_time_of_day: bool,
            /// If a time is missing (or `keep_time_of_day == false`), use this time if provided.
            pub default_time: Option<NaiveTime>,
            /// Preserve `<active>` vs `[inactive]` brackets from the source.
            pub preserve_active: bool,
            /// Reference date used when `ShiftByDeltaDays` is selected.
            pub shift_from: Option<NaiveDate>,
        }

        impl Default for ReschedulePolicy {
            fn default() -> Self {
                Self {
                    scheduled_rule: RescheduleRule::SetToTarget,
                    deadline_rule: RescheduleRule::ToTargetIfOverdue,
                    keep_time_of_day: true,
                    default_time: None,
                    preserve_active: true,
                    shift_from: None,
                }
            }
        }

        /* ------------------------------ Public API ------------------------------ */

        /// Build a new journal entry from a template and a collection of parsed journal files.
        ///
        /// Default policy (if you don't need custom behavior):
        /// - SCHEDULED => set to target date
        /// - DEADLINE => set to target date only if overdue
        /// - Keep time-of-day, keep active/inactive brackets
        pub fn build_from_files<'a>(
            template: &OrgFile,
            journal_files: impl IntoIterator<Item = &'a OrgFile>,
            date: NaiveDate,
            verbose: bool,
        ) -> OrgFile {
            build_from_files_with_policy(
                template,
                journal_files,
                date,
                ReschedulePolicy::default(),
                verbose,
            )
        }

        /// Same as `build_from_files` but with an explicit rescheduling policy.
        pub fn build_from_files_with_policy<'a>(
            template: &OrgFile,
            journal_files: impl IntoIterator<Item = &'a OrgFile>,
            date: NaiveDate,
            policy: ReschedulePolicy,
            verbose: bool,
        ) -> OrgFile {
            let mut new_file = clone_as_new_file(template);

            if new_file.title.is_none() {
                new_file.title = Some(date.to_string());
            }

            // Collect from all files, dedupe on (path_key, todo_title_key)
            let mut seen: BTreeSet<(Vec<String>, String)> = BTreeSet::new();
            let mut buckets: BucketTree = BucketTree::default();

            for jf in journal_files {
                if verbose {
                    eprintln!("Projecting journal file {:?}", jf.path);
                }
                let mut path = Vec::<String>::new();
                for h in &jf.headings {
                    collect_incomplete_todos(
                        jf,
                        h,
                        &mut path,
                        &mut buckets,
                        &mut seen,
                        date,
                        &policy,
                        verbose,
                    );
                }
            }

            // Merge bucketed TODOs into new file.
            let mut roots = std::mem::take(&mut new_file.headings);
            for (path_vec, todos) in buckets.into_flat_vec() {
                let parent = ensure_path(&mut roots, &path_vec);
                merge_todos(parent, todos);
            }
            new_file.headings = roots;

            let formatted = format_org_file(&new_file);
            let mut repro = parse_org_from_str(new_file.path.clone(), &formatted)
                .expect("formatted journal entry should parse");
            repro.id = new_file.id;
            transplant_ids(&new_file.headings, &mut repro.headings);
            repro.title = new_file.title.clone();
            repro.file_tags = new_file.file_tags.clone();
            repro.settings = new_file.settings.clone();
            repro.path = new_file.path.clone();

            repro
        }

        /// Build from a workspace and a journal directory (relative to workspace root).
        /// Uses only Loaded files (pure; no I/O here).
        pub fn build_from_workspace(
            template: &OrgFile,
            ws: &OrgWorkspace,
            journal_dir: &RelPath,
            date: NaiveDate,
            verbose: bool,
        ) -> OrgFile {
            build_from_workspace_with_policy(
                template,
                ws,
                journal_dir,
                date,
                ReschedulePolicy::default(),
                verbose,
            )
        }

        pub fn build_from_workspace_with_policy(
            template: &OrgFile,
            ws: &OrgWorkspace,
            journal_dir: &RelPath,
            date: NaiveDate,
            policy: ReschedulePolicy,
            verbose: bool,
        ) -> OrgFile {
            let mut parsed: Vec<&OrgFile> = Vec::new();

            if let Some(dir) = ws.root.find_dir(journal_dir) {
                let mut entries = Vec::new();
                dir.collect_files(&mut entries);
                for e in entries {
                    if let Some(f) = e.loaded() {
                        parsed.push(f);
                    }
                }
            }

            build_from_files_with_policy(template, parsed, date, policy, verbose)
        }

        /* ------------------------------ Internals ------------------------------ */

        /// A small bucketed tree: path (Vec<String>) → Vec<Heading> (TODO nodes).
        #[derive(Default)]
        struct BucketTree {
            map: IndexMap<Vec<String>, Vec<Heading>>,
        }
        impl BucketTree {
            fn push(&mut self, path: Vec<String>, h: Heading) {
                self.map.entry(path).or_default().push(h);
            }
            fn into_flat_vec(self) -> Vec<(Vec<String>, Vec<Heading>)> {
                self.map.into_iter().collect()
            }
        }

        fn collect_incomplete_todos(
            file: &OrgFile,
            h: &Heading,
            path: &mut Vec<String>,
            buckets: &mut BucketTree,
            seen: &mut BTreeSet<(Vec<String>, String)>,
            target_date: NaiveDate,
            policy: &ReschedulePolicy,
            verbose: bool,
        ) {
            if verbose {
                eprintln!(
                    "Collecting TODOs: file {:?}, heading {:?}",
                    file.path,
                    h.title.plain_text()
                );
            }
            let this_title = h.title.plain_text();
            let use_as_group = !looks_like_date_heading(&this_title) || !path.is_empty();
            if use_as_group {
                path.push(this_title.clone());
            }

            if is_incomplete_todo(h, &file.settings) {
                let key_path = normalized_path(path);
                let title_key = normalize(&h.title.plain_text());
                let dedupe_key = (key_path.clone(), title_key.clone());
                if !seen.contains(&dedupe_key) {
                    seen.insert(dedupe_key);

                    // Clone & strip children; reschedule planning in place according to policy.
                    let mut copy = h.clone();
                    copy.children.clear();
                    reschedule_planning_in_place(&mut copy.planning, target_date, policy);
                    scrub_heading_sources(&mut copy);

                    buckets.push(key_path, copy);
                }
            }

            for c in &h.children {
                collect_incomplete_todos(
                    file,
                    c,
                    path,
                    buckets,
                    seen,
                    target_date,
                    policy,
                    verbose,
                );
            }

            if use_as_group {
                path.pop();
            }
        }

        fn is_incomplete_todo(h: &Heading, settings: &FileSettings) -> bool {
            let Some(todo) = &h.todo else {
                return false;
            };
            if todo.is_done {
                return false;
            }
            let done_words = compute_done_keywords(settings);
            !done_words.contains(&todo.text)
        }

        fn compute_done_keywords(settings: &FileSettings) -> BTreeSet<String> {
            let mut out = BTreeSet::new();
            for seq in &settings.todo_sequences {
                let mut done = false;
                for item in &seq.items {
                    if item == "|" {
                        done = true;
                        continue;
                    }
                    if done {
                        out.insert(item.to_string());
                    }
                }
            }
            if out.is_empty() {
                for s in ["DONE", "CANCELLED", "CANCELED", "ABORTED", "VOID"] {
                    out.insert(s.to_string());
                }
            }
            out
        }

        fn looks_like_date_heading(title: &str) -> bool {
            let t = title.trim();
            if t.len() < 10 {
                return false;
            }
            let (y, _rest) = t.split_at(4);
            y.chars().all(|c| c.is_ascii_digit())
                && t.get(4..5) == Some("-")
                && t.get(5..7)
                    .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(false)
                && t.get(7..8) == Some("-")
                && t.get(8..10)
                    .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(false)
        }

        fn normalized_path(path: &[String]) -> Vec<String> {
            path.iter().map(|s| normalize(s)).collect()
        }

        fn normalize(s: &str) -> String {
            let mut out = String::with_capacity(s.len());
            let mut prev_space = false;
            for ch in s.chars() {
                let lc = ch.to_ascii_lowercase();
                if lc.is_whitespace() {
                    if !prev_space {
                        out.push(' ');
                        prev_space = true;
                    }
                } else {
                    out.push(lc);
                    prev_space = false;
                }
            }
            out.trim().to_string()
        }

        fn scrub_heading_sources(h: &mut Heading) {
            h.mark_headline_dirty();
            h.mark_planning_dirty();
            h.mark_properties_dirty();
            h.mark_logbook_dirty();
            for block in &mut h.section.blocks {
                block.mark_dirty();
            }
            for child in &mut h.children {
                scrub_heading_sources(child);
            }
        }

        fn transplant_ids(src: &[Heading], dst: &mut [Heading]) {
            assert_eq!(src.len(), dst.len());
            for (s, d) in src.iter().zip(dst.iter_mut()) {
                d.id = s.id;
                d.canonical_id = s.canonical_id.clone();
                transplant_ids(&s.children, &mut d.children);
            }
        }

        fn clone_as_new_file(template: &OrgFile) -> OrgFile {
            let mut f = template.clone();
            f.id = OrgFileId(Uuid::new_v4());
            f.path = None;
            f
        }

        /// Ensure a heading path exists under `roots` and return the last node.
        fn ensure_path<'a>(roots: &'a mut Vec<Heading>, path: &[String]) -> &'a mut Heading {
            let use_path = if path.is_empty() {
                vec!["tasks".to_string()]
            } else {
                path.to_vec()
            };
            let mut slice: &mut Vec<Heading> = roots;
            let mut level: u8 = 1;
            for component in &use_path {
                let key = normalize(component);
                let mut idx = None;
                for (pos, h) in slice.iter().enumerate() {
                    if normalize(&h.title.plain_text()) == key {
                        idx = Some(pos);
                        break;
                    }
                }
                if idx.is_none() {
                    let mut h = Heading::new(
                        level.min(8),
                        RichText {
                            inlines: vec![Inline::Text(component.clone())],
                        },
                    );
                    h.todo = None;
                    h.priority = None;
                    slice.push(h);
                    idx = Some(slice.len() - 1);
                }
                let pos = idx.unwrap();
                if slice[pos].level != level.min(8) {
                    slice[pos].level = level.min(8);
                }
                let ptr: *mut Heading = &mut slice[pos];
                unsafe {
                    slice = &mut (*ptr).children;
                }
                level = level.saturating_add(1);
            }
            get_mut_by_path(roots, &use_path).expect("path must exist")
        }

        fn get_mut_by_path<'a>(
            roots: &'a mut [Heading],
            path: &[String],
        ) -> Option<&'a mut Heading> {
            if path.is_empty() {
                return None;
            }
            let mut slice: &mut [Heading] = roots;
            let mut found: *mut Heading = std::ptr::null_mut();
            for component in path {
                let key = normalize(component);
                let mut hit: Option<*mut Heading> = None;
                for h in slice {
                    if normalize(&h.title.plain_text()) == key {
                        hit = Some(h as *mut Heading);
                        break;
                    }
                }
                let Some(ptr) = hit else {
                    return None;
                };
                found = ptr;
                unsafe {
                    slice = &mut (*ptr).children;
                }
            }
            if found.is_null() {
                None
            } else {
                unsafe { Some(&mut *found) }
            }
        }

        fn merge_todos(parent: &mut Heading, mut todos: Vec<Heading>) {
            for mut todo in todos.drain(..) {
                scrub_heading_sources(&mut todo);
                let key = normalize(&todo.title.plain_text());
                if let Some(existing_idx) = parent
                    .children
                    .iter()
                    .position(|h| normalize(&h.title.plain_text()) == key)
                {
                    let existing = &mut parent.children[existing_idx];
                    if existing.todo.is_none() && todo.todo.is_some() {
                        existing.todo = todo.todo.take();
                        existing.mark_headline_dirty();
                    }
                    if existing.priority.is_none() && todo.priority.is_some() {
                        existing.priority = todo.priority;
                        existing.mark_headline_dirty();
                    }
                    if !todo.tags.is_empty() {
                        existing.tags.extend(todo.tags.into_iter());
                        existing.mark_headline_dirty();
                    }
                    if existing.planning.scheduled.is_none() && todo.planning.scheduled.is_some() {
                        existing.planning.scheduled = todo.planning.scheduled.take();
                        existing.mark_planning_dirty();
                    }
                    if existing.planning.deadline.is_none() && todo.planning.deadline.is_some() {
                        existing.planning.deadline = todo.planning.deadline.take();
                        existing.mark_planning_dirty();
                    }
                    if existing.planning.closed.is_none() && todo.planning.closed.is_some() {
                        existing.planning.closed = todo.planning.closed.take();
                        existing.mark_planning_dirty();
                    }
                    existing
                        .section
                        .blocks
                        .extend(todo.section.blocks.into_iter());
                    for (k, v) in todo.properties.props.into_iter() {
                        if !existing.properties.props.contains_key(&k) {
                            existing.properties.props.insert(k, v);
                            existing.mark_properties_dirty();
                        }
                    }
                    if !todo.logbook.clock.is_empty() || !todo.logbook.raw.is_empty() {
                        existing.mark_logbook_dirty();
                    }
                    existing
                        .logbook
                        .clock
                        .extend(todo.logbook.clock.into_iter());
                    existing.logbook.raw.extend(todo.logbook.raw.into_iter());
                } else {
                    todo.level = parent.level.saturating_add(1).min(8);
                    parent.children.push(todo);
                }
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;
            use crate::{format::format_org_file, parse_org_from_str};
            use chrono::NaiveDate;

            #[test]
            fn newly_built_entry_formats_stably() {
                let template =
                    parse_org_from_str(None, "* TODO Template\n").expect("template parse");
                let journal = parse_org_from_str(None, "* TODO Carry\nSCHEDULED: <2025-02-01>\n")
                    .expect("journal parse");

                let entry = build_from_files(
                    &template,
                    [&journal],
                    NaiveDate::from_ymd_opt(2025, 2, 2).unwrap(),
                    false,
                );

                let formatted1 = format_org_file(&entry);
                let formatted2 = format_org_file(&entry);
                assert_eq!(formatted1, formatted2);
                if let Some(src) = &entry.source_text {
                    assert_eq!(src, &formatted2);
                } else {
                    panic!("expected source_text to be populated");
                }
            }
        }

        /* ----------------------- Rescheduling implementation ---------------------- */

        fn reschedule_planning_in_place(
            p: &mut Planning,
            target: NaiveDate,
            policy: &ReschedulePolicy,
        ) {
            if let Some(ts) = p.scheduled.clone() {
                p.scheduled = Some(reschedule_ts(&ts, target, policy, policy.scheduled_rule));
            }
            if let Some(ts) = p.deadline.clone() {
                p.deadline = Some(reschedule_ts(&ts, target, policy, policy.deadline_rule));
            }
            // CLOSED is intentionally not touched for carried-over incomplete tasks.
        }

        fn reschedule_ts(
            ts: &Timestamp,
            target: NaiveDate,
            policy: &ReschedulePolicy,
            rule: RescheduleRule,
        ) -> Timestamp {
            match rule {
                RescheduleRule::NoChange => ts.clone(),
                RescheduleRule::SetToTarget => rewrite_to_target(ts, target, policy),
                RescheduleRule::ToTargetIfOverdue => {
                    if ts.date < target {
                        rewrite_to_target(ts, target, policy)
                    } else {
                        ts.clone()
                    }
                }
                RescheduleRule::ShiftByDeltaDays => {
                    let Some(from) = policy.shift_from else {
                        return ts.clone();
                    };
                    let delta = (target - from).num_days();
                    if delta == 0 {
                        return ts.clone();
                    }
                    shift_by_days(ts, delta, policy)
                }
            }
        }

        fn rewrite_to_target(
            ts: &Timestamp,
            target: NaiveDate,
            policy: &ReschedulePolicy,
        ) -> Timestamp {
            let mut out = ts.clone();
            // Date
            let old_date = out.date;
            out.date = target;

            // Time
            out.time = match (policy.keep_time_of_day, ts.time, policy.default_time) {
                (true, Some(t), _) => Some(t),
                (true, None, Some(def)) => Some(def),
                (true, None, None) => None,
                (false, _, Some(def)) => Some(def),
                (false, _, None) => None,
            };

            // Preserve/normalize active flag
            if !policy.preserve_active {
                out.active = true;
            }

            // End range: keep duration in days if end has an explicit date; otherwise keep end time as-is.
            if let Some(end) = &mut out.end {
                if let Some(ed) = end.date {
                    let day_span = (ed - old_date).num_days();
                    end.date = Some(target + Duration::days(day_span));
                }
                // if end.time is Some but date is None, it's a same-day time range; keep it as-is.
            }

            out
        }

        fn shift_by_days(ts: &Timestamp, delta_days: i64, policy: &ReschedulePolicy) -> Timestamp {
            let mut out = ts.clone();
            out.date = ts.date + Duration::days(delta_days);

            // If not keeping original time-of-day, apply default time if provided.
            if !policy.keep_time_of_day {
                out.time = policy.default_time;
            } else if out.time.is_none() {
                // Keeping time but there is none; optionally fill default time.
                if let Some(def) = policy.default_time {
                    out.time = Some(def);
                }
            }

            if !policy.preserve_active {
                out.active = true;
            }

            if let Some(end) = &mut out.end {
                if let Some(ed) = end.date {
                    end.date = Some(ed + Duration::days(delta_days));
                }
            }

            out
        }
    }
}

pub use format::format_org_file;
pub use parser::{NomOrgParser, parse_org_from_str};
