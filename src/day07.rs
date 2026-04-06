// Advent of Code 2025, Day 7: Beam Splitter
// https://adventofcode.com/2025/day/7
//
// A beam falls downward through a grid from a start column (S). Empty cells
// pass the beam straight down; splitter cells (^) stop the beam and spawn two
// new beams going left and right. Part 1: count how many splitters are hit.
// Part 2: count how many distinct "timelines" (paths) exit the bottom of the
// grid, computed via bottom-up DP.

use std::{collections::VecDeque, fs};

fn parse(filename: &str) -> (Vec<Vec<char>>, usize) {
    let data = fs::read_to_string(filename).unwrap();
    let grid: Vec<Vec<char>> = data.lines().map(|l| l.chars().collect()).collect();
    let s_col = grid[0].iter().position(|&c| c == 'S').unwrap();
    (grid, s_col)
}

// Simulate beams traveling downward from (0, start_col). When a beam hits a
// splitter (^) it stops and spawns two new beams at (same row, col±1). Beams
// that revisit a cell are discarded. Returns the number of splits (each
// splitter hit counts once per unique beam path reaching it).
fn count_splits(grid: &[Vec<char>], start_col: usize) -> i32 {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut visited = vec![vec![false; cols]; rows];
    let mut queue = VecDeque::new();
    let mut splits = 0;

    queue.push_back((0usize, start_col));
    while let Some((r, c)) = queue.pop_front() {
        if r >= rows || c >= cols || visited[r][c] {
            continue;
        }
        visited[r][c] = true;
        if grid[r][c] == '^' {
            splits += 1;
            if c > 0     { queue.push_back((r, c - 1)); }
            if c + 1 < cols { queue.push_back((r, c + 1)); }
        } else {
            if r + 1 < rows { queue.push_back((r + 1, c)); }
        }
    }
    splits
}

pub fn solve_part1(filename: &str) -> i32 {
    let (grid, s_col) = parse(filename);
    count_splits(&grid, s_col)
}

pub fn solve_part2(filename: &str) -> i64 {
    let (grid, s_col) = parse(filename);
    let rows = grid.len();
    let cols = grid[0].len();

    // dp[r][c] = number of distinct timelines for a beam starting at (r, c).
    // Exit (bottom or side): 1. Empty cell: dp[r+1][c]. Splitter: dp[r][c-1] + dp[r][c+1].
    //
    // Two passes per row (bottom-up) avoid any ordering issues: non-splitters
    // first (dp[r+1][c] is already known), then splitters (whose neighbours are
    // guaranteed non-splitters, so their row-r values are already set).
    let mut dp = vec![vec![1i64; cols]; rows + 1]; // row `rows` = exit = 1

    for r in (0..rows).rev() {
        for c in 0..cols {
            if grid[r][c] != '^' {
                dp[r][c] = dp[r + 1][c];
            }
        }
        for c in 0..cols {
            if grid[r][c] == '^' {
                let left  = if c == 0        { 1 } else { dp[r][c - 1] };
                let right = if c + 1 >= cols { 1 } else { dp[r][c + 1] };
                dp[r][c] = left + right;
            }
        }
    }

    dp[0][s_col]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
.......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............
";

    #[test]
    fn example() {
        let tmp = crate::testutil::TempFile::write(EXAMPLE);
        assert_eq!(solve_part1(tmp.path()), 21);
        assert_eq!(solve_part2(tmp.path()), 40);
    }
}
