// Advent of Code 2025, Day 1: Rotary Lock
// https://adventofcode.com/2025/day/1
//
// A dial with 100 positions (0–99) starts at 50. Each instruction rotates it
// left or right by some number of clicks. Part 1: count how many instructions
// land the dial exactly on 0 after rotation. Part 2: count how many individual
// clicks land on 0 across all instructions (i.e. the dial passes through 0
// during rotation, not just at the end).
//
// Part 1: simulate dial rotations mod 100, count times it lands on 0. O(instructions).
// Part 2: naive simulation is O(total clicks). Instead, compute zero-crossings per
// rotation analytically in O(1): given start position s and delta ±1, the first click
// that hits 0 is k0 = (-s*delta) mod 100 (substituting 100 if 0), then hits repeat
// every 100 clicks, so count = (clicks - k0) / 100 + 1 if clicks >= k0, else 0.

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

const PERIOD: i32 = 100;

fn dir_to_delta(c: char) -> i32 {
    if c == 'L' { -1 } else { 1 }
}

pub fn solve_part1(filename: &str) -> i32 {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut dial = 50;
    let mut zerocount = 0;
    for line in reader.lines().map_while(Result::ok) {
        let delta = dir_to_delta(line.chars().next().unwrap());
        let clicks = line[1..].parse::<i32>().unwrap();
        dial = (dial + PERIOD + clicks * delta) % PERIOD;
        if dial == 0 {
            zerocount += 1;
        }
    }
    zerocount
}

// Count how many times the dial hits 0 during `clicks` steps of size `delta`,
// starting from position `start` (checking after each click, not before).
//
// We want #{k ∈ [1, clicks] : (start + k*delta) ≡ 0 (mod 100)}.
//
// Rearranging: k ≡ -start*delta (mod 100). The first valid k is:
//   k0 = (-start * delta).rem_euclid(100), or 100 if that is 0.
//
// Then hits occur at k0, k0+100, k0+200, ... so the count is:
//   0                         if clicks < k0
//   (clicks - k0) / 100 + 1  otherwise
//
// Example: start=50, R1000 (delta=+1) → k0=50, count=(1000-50)/100+1=10. ✓
fn count_zeros(start: i32, delta: i32, clicks: i32) -> i32 {
    let k0 = (-start * delta).rem_euclid(PERIOD);
    let k0 = if k0 == 0 { PERIOD } else { k0 };
    if clicks < k0 { 0 } else { (clicks - k0) / PERIOD + 1 }
}

pub fn solve_part2(filename: &str) -> i32 {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut dial = 50;
    let mut zerocount = 0;
    for line in reader.lines().map_while(Result::ok) {
        let delta = dir_to_delta(line.chars().next().unwrap());
        let clicks = line[1..].parse::<i32>().unwrap();
        zerocount += count_zeros(dial, delta, clicks);
        dial = (dial + PERIOD + clicks * delta) % PERIOD;
    }
    zerocount
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
L68
L30
R48
L5
R60
L55
L1
L99
R14
L82
";

    #[test]
    fn example() {
        let tmp = crate::testutil::TempFile::write(EXAMPLE);
        assert_eq!(solve_part1(tmp.path()), 3);
        assert_eq!(solve_part2(tmp.path()), 6);
    }
}
