# Repository Guidelines

## Project Structure & Module Organization
- `backend/` is the Rust workspace member and primary crate.
- `backend/src/main.rs` contains the current CLI entry point and map parsing logic.
- `data/` holds local map inputs (e.g., `data/bengaluru.osm.pbf`) used by the binary.
- `target/` is Cargo build output (generated).

## Build, Test, and Development Commands
- `cargo build -p backend` builds the backend crate.
- `cargo run -p backend` runs the CLI (expects `data/bengaluru.osm.pbf` to exist).
- `cargo test -p backend` runs tests (none currently).
- `cargo check -p backend` does a fast compile check during development.

## Coding Style & Naming Conventions
- Rust 2024 edition is in use; follow standard Rust formatting.
- Indentation: 4 spaces (rustfmt defaults).
- Naming: `snake_case` for functions/variables, `UpperCamelCase` for types, `SCREAMING_SNAKE_CASE` for constants.
- If you add formatting or linting, prefer `cargo fmt` and `cargo clippy` with default settings.

## Testing Guidelines
- No test framework is configured beyond Rust’s built-in test harness.
- Place unit tests in `backend/src` modules using `#[cfg(test)]`.
- For integration tests, use `backend/tests/` (e.g., `backend/tests/parse_osm.rs`).

## Commit & Pull Request Guidelines
- Git history currently shows only an “Initial Commit”; no established message convention yet.
- Use clear, imperative commit subjects (e.g., “Add graph construction from OSM ways”).
- PRs should include a short summary, the commands run (e.g., `cargo build -p backend`), and any data prerequisites or sample inputs used.

## Configuration Tips
- The binary expects a local `.osm.pbf` file at `data/bengaluru.osm.pbf` by default. Update the path in `backend/src/main.rs` if you use a different dataset.
