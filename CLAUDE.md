# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run all solutions (benchmarked)
cargo run

# Run a specific day/part
cargo run -- run 12 1

# Run tests
cargo test

# Run tests for a specific day
cargo test day01

# Check without building
cargo check

# Lint
cargo clippy
```

## Structure

Each day's solution lives in `src/dayNN.rs` and is registered as a module in `src/main.rs`. Solutions are benchmarked and displayed in a table; each runs repeatedly up to a 5-second timeout.

Each day module exposes:
- `pub fn solve_part1(filename: &str) -> impl Display`
- `pub fn solve_part2(filename: &str) -> impl Display`

Puzzle inputs go in `inputs/dayNN.txt`. Puzzle descriptions are cached in `inputs/dayNN.puzzle.txt` and can be downloaded with `cargo run -- puzzle NN`.

## Tests

Each day has an `example` test in its `#[cfg(test)]` module using the sample input from the puzzle description. Example data is stored as a `const EXAMPLE: &str` and written to a temporary file via `crate::testutil::TempFile::write(EXAMPLE)`, which deletes the file on drop.

Do not include correct puzzle answers in commit messages.

## Adding a New Day

1. Create `src/dayNN.rs` with `pub fn solve_part1` and `pub fn solve_part2`
2. Add `mod dayNN;` in `src/main.rs`
3. Wire up `ensure_input`, `expected_answers`, and solution entries in `run_solutions()`
4. Add an `example` test using `testutil::TempFile`
