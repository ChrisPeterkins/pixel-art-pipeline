# Pixelpipe — Project Conventions

## Rust Style & Principles

### Error Handling
- Use `thiserror` for all error types in the library crate (`pixelpipe-core`)
- Propagate errors with `?` — avoid `.unwrap()` outside of tests
- Return `Result<T>` using our custom `error::Result` alias
- Error messages should include context (file paths, sprite names, palette names)

### Ownership & Borrowing
- Prefer `&str` over `&String` in function parameters
- Prefer borrowing (`&T`) over cloning — only clone when ownership is truly needed
- Use `Cow<str>` when a function may or may not need to allocate
- Avoid unnecessary `.to_string()` / `.clone()` chains

### Naming
- Types: `PascalCase` (e.g., `SheetResult`, `PipelineContext`)
- Functions/methods: `snake_case` (e.g., `run_pipeline`, `enforce_nearest`)
- Constants: `SCREAMING_SNAKE_CASE`
- Module files: `snake_case.rs`
- Enum variants: `PascalCase`, use `#[serde(rename_all = "lowercase")]` for config

### Module Organization
- One responsibility per module — if a file exceeds ~300 lines, consider splitting
- Public API goes through `mod.rs` re-exports
- Keep `pub` surface minimal — only expose what other crates need
- Place unit tests in `#[cfg(test)] mod tests` at bottom of same file
- Place integration tests in `crates/pixelpipe-core/tests/`

### Functions
- Keep functions under ~50 lines — extract helpers for clarity
- Use early returns for guard clauses
- Prefer iterators and combinators (`.map()`, `.filter()`, `.collect()`) over manual loops where they improve readability — but don't force it when a `for` loop is clearer
- Use `impl Iterator` return types over collecting into a Vec when the caller just iterates

### Structs & Types
- Derive `Debug` on all public types
- Derive `Clone` only when cloning is expected
- Use `#[serde(deny_unknown_fields)]` on all config structs to catch typos
- Use `#[serde(default)]` with explicit default functions — don't rely on `Default` impl alone
- Prefer struct fields over long parameter lists (builder pattern for 4+ params)

### Testing
- Every module gets unit tests for core logic
- Integration tests verify end-to-end pipeline behavior
- Test fixtures go in the `pixelpipe-test-fixtures` crate
- Use `assert_eq!` with descriptive messages for pixel-level comparisons
- Clean up temp directories in tests

### Formatting & Linting
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- No `#[allow(unused)]` without a comment explaining why

### Performance
- Use `rayon` for parallelizing image I/O and processing (Milestone 7)
- Never use any scaling filter other than `FilterType::Nearest`
- Prefer in-place mutation over allocation when processing large images

## Project Architecture

- `pixelpipe-core`: Library crate with all pipeline logic — must stay CLI-independent for future WASM/editor reuse
- `pixelpipe-cli`: Thin binary crate — parses args, loads config, calls core, reports results
- Pipeline phases implement the `PipelinePhase` trait and communicate via `PipelineContext`

## Build & Test

```bash
cargo fmt --all                  # format
cargo clippy --workspace         # lint
cargo test --workspace           # all tests
cargo run -- build               # run pipeline
cargo run -- validate            # check config
```
