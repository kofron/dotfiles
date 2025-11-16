# 2025-02-14 — Org crate kickoff

- **Principles**: Sourcing from *Simplicity* and *Reuse*, we will treat `org` as a reusable library + CLI binary. The domain core remains pure data (Purity) so higher layers stay flexible.
- **Workspace reality**: GUIDE.md already contains richly sketched modules (`core`, `journal`, `agenda`, `storage`, workspace/projectors). We will transplant that code into proper modules rather than reinventing it.
- **Plan highlights**:
  - carve out a library crate layout (`org/src/lib.rs`) exposing domain modules, while keeping `src/main.rs` as the thin clap-based CLI.
  - port existing GUIDE snippets verbatim where possible, then normalize naming/tests to match Rust workspace conventions.
  - introduce adapters (parser, workspace loader) as separate modules to keep the functional core isolated, backing them with Rust unit/integration tests located alongside modules plus any cross-crate tests.
  - future CLI commands will compose pure operations (parse → project → print) with minimal side effects.
- **Testing posture**: every ported component arrives with targeted tests; parsing and projection code get golden tests. We'll lean on `cargo test` frequently and add `.test.ts` suites only if/when we surface TypeScript bindings.

This entry seeds the design diary; future adjustments will append rather than rewrite, preserving decision history per *Code is communication*.

# 2025-02-14 — Parser, projectors, and CLI

- **Parser uplift**: Ported the nom parser from `GUIDE.md`, including the inline-markup upgrade (emphasis, links, code, etc.). `rt_text` now delegates to `parse_inlines_str`, keeping RichText generation pure and reusable. Added regression tests directly in the module to keep the functional core well-guarded (*Tests are everything*).
- **Workspace & projectors**: Brought across agenda and journal projectors untouched so downstream crates tap the same pure logic. No mutation leaks into the library; everything returns new values or projections. This honours *Purity* and maximises *Reuse*.
- **Serde quirks**: `FixedOffset` needed a small wrapper to stay serialisable. Implemented a dedicated `serde_fixed_offset_opt` helper to preserve round-trip fidelity without bending the domain types.
- **CLI shell**: Replaced the hello-world binary with a Clap-based interface (`parse`, `agenda`, `journal new`). The commands orchestrate the pure library:
  - `parse` pipes NomOrgParser output to either debug or JSON.
  - `agenda` stitches projected agenda items, with optional JSON and date filters.
  - `journal new` carries incomplete TODOs forward using the projector and emits JSON (writing to a file when requested).
  All side-effects (filesystem, stdout) live here, keeping the library untouched (*functional core, imperative shell*).
- **JSON output**: Leaned on `serde_json` for CLI output until we grow a pretty-printer back to `.org`. Kept the dependency surface minimal to stay within *Simplicity*.

# 2025-02-14 — Formatter wiring & CLI polish

- **Formatter integration**: Captured source ranges throughout the parser so `format_org_file` can reuse untouched text while canonicalising new segments. Journal projectors reparse their output post-format and transplant IDs to guarantee an immediate round-trip (*Purity* + *Tests are everything*).
- **CLI upgrades**: Added an explicit `format` subcommand and taught `journal new` to emit canonical `.org` via the formatter (or JSON when requested). Fresh journal files now arrive ready for on-disk writes without extra tooling.
- **Testing**: Extended formatter coverage with stability assertions and ensured the journal projector produces formatter-stable output, keeping regressions visible.
- **Agenda quality-of-life**: CLI accepts directories for every command, and agenda mode now supports `--include-todos` to surface undated tasks alongside scheduled items, matching Org agenda expectations while retaining deterministic ordering.

# 2025-02-15 — Journal TODO merge fix

- **Bug**: Carrying TODOs forward produced duplicate heading stacks (`** pay VM` > `*** TODO pay VM`). We were normalising the grouping path with the dangling TODO headline included, so `ensure_path` synthesised an intermediate heading.
- **Fix**: To stay inside *Simplicity* and preserve the existing merge projector (*Reuse*), we now bucket TODOs using their parent path, leaving the TODO node itself to be merged by `merge_todos`. The traversal still pushes the headline for child context, satisfying *Purity* by keeping the projector deterministic.
- **Tests**: Added a regression case in `org/src/lib.rs` ensuring the carried-over children under "* Plan for today" remain TODO nodes directly—anchoring the bugfix in the suite (*Tests are everything*).

# 2025-02-15 — Journal CLI write automation

- **Feature**: Extended `journal-new` to accept file/directory inputs directly and resolve the lowest common journal directory so `--write` can auto-drop a `{YYYY-MM-DD}.org` entry exactly where the source material lives, keeping with *Simplicity*.
- **Implementation**: Reused the shared `expand_inputs` walker and layered a pure `resolve_write_directory` helper to compute targets without touching IO besides canonicalisation (*Reuse*, *Purity*).
- **CLI ergonomics**: Added a conflict guard between `--write` and `--output`, defaulted `--date` to today, and left existing JSON/Org emit paths intact so downstream scripts keep working.
- **Tests**: Backstopped the directory resolution logic with tempdir-backed unit tests to lock in the lowest-common-ancestor behaviour (*Tests are everything*).

# 2025-02-15 — Journal CLI idempotence

- **Behaviour**: When `--write` encounters an existing `{date}.org`, we now reuse the on-disk entry as the template and overwrite in place, keeping the projector pure while making repeat runs a no-op (*Purity*, *Simplicity*).
- **Projector guardrail**: Added a regression test proving that feeding the newly generated entry back into `build_from_files` yields identical output, cementing the idempotent contract (*Tests are everything*).
