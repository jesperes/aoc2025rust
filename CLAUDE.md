# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run all solutions (benchmarked)
cargo run

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

Each day's solution lives in `src/dayNN.rs` and is registered as a module in `src/main.rs`. The `main.rs` runs all solutions via a `benchmark()` wrapper that times each call and prints results.

Each day module exposes two public functions:
- `solve_part1(filename: &str) -> impl Display`
- `solve_part2(filename: &str) -> impl Display`

Puzzle inputs go in `inputs/dayNN.txt`.

## Adding a New Day

1. Create `src/dayNN.rs` with `pub fn solve_part1` and `pub fn solve_part2`
2. Add `mod dayNN;` in `src/main.rs`
3. Add two `benchmark(...)` calls in `main()`
4. Place input in `inputs/dayNN.txt`
