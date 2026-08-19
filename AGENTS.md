# AGENTS.md

## Build

- Library only: `cargo build --release`
- Binaries (`cjval` + `cjvalext`): `cargo build --release --features build-binary`
- The `build-binary` feature gates all CLI/TUI/HTTP deps. The library compiles without it.
- Docker: `docker/Dockerfile` (multi-stage; published to `tudelft3d/cjval` on release via `.github/workflows/image_build.yml`)

## Test

- `cargo test` — 44 tests across 14 test files in `tests/`, run from project root
- Tests use relative paths to `data/` and `schemas/` from project root
- Run a single test: `cargo test --test extensions extension_missing_url`
- No CI test suite — the only workflow builds Docker on release

## Architecture

- **Library** `src/lib.rs` — `CJValidator` struct: `from_str()`, `validate()` → `IndexMap<String, ValSummary>`
- **Binaries** `src/bin/cjval.rs` (CityJSON/Seq validator with TUI) and `src/bin/cjvalext.rs` (extension file validator)
- CityJSON schemas are bundled at compile time via `include_str!()` from `schemas/10/`, `schemas/11/`, `schemas/20/`
- Supported CityJSON versions: 1.0, 1.1, 2.0
- Extension schemas auto-downloaded from URLs in the `"extensions"` property; override with `-e`

### JSON report

- `CJValidator::get_report(file: &str) -> CJReport` — validates and returns a serializable report
- Structs: `CJReport`, `CheckResults` (errors + warnings maps), `ValSummary` (all derive `Serialize`)
- `ValSummary` fields are `pub`: `status` (serialized as `"valid"`), `errors`, `warning` (serialized as `#[serde(skip)]`)
- CLI: `--report` outputs JSON to stdout (suppresses TUI), `--summary` prints one-line result (was `--quiet`)
- Seq mode with `--report` outputs JSONL (one object per line)
- The binary has its own internal `ValidationResult` struct — be careful of name collisions with new lib types

## Style

- No `rustfmt.toml`, `clippy.toml`, or pre-commit hooks — just `cargo check`

## Data

- Test fixtures in `data/` (33 cityjson/jsonl files)
- Extension schema fixtures in `schemas/extensions/`
