// Advent of Code 2025, Day 2: ID Validator
// https://adventofcode.com/2025/day/2
//
// Given ranges of numeric IDs, find the sum of "invalid" IDs — numbers whose
// decimal representation consists of a shorter pattern repeated two or more
// times (e.g. 1212, 99, 123123). Part 1 counts IDs with exactly one repeated
// pattern (m=2). Part 2 counts IDs with any number of repetitions (m≥2).
//
// Both parts avoid iterating over individual IDs in each range.
//
// Part 1: invalid IDs (pattern P repeated exactly twice) have the form N = P * (10^k + 1).
// For each k, valid P values in a range form a contiguous interval, summed as an
// arithmetic series in O(1) per range per k.
//
// Part 2: extends to m >= 2 repetitions where N = P * (10^(km)-1)/(10^k-1). Different
// (k,m) pairs with the same total digit length L can generate the same number (e.g.
// 1111 = 11*101 and 1*1111). Inclusion-exclusion over the proper divisors of L, using
// gcd to collapse intersections, gives the exact sum with O(R * L * 2^d) operations
// (R ranges, L <= 10 digit lengths, d <= 3 proper divisors per L).

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn parse_ranges(filename: &str) -> Vec<(i64, i64)> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let line = reader.lines().next().unwrap().unwrap();
    line.split(',')
        .map(|r| {
            let (a, b) = r.split_once('-').unwrap();
            (a.parse().unwrap(), b.parse().unwrap())
        })
        .collect()
}

// An invalid ID is a number formed by repeating some k-digit pattern P two or
// more times. For example: 99 (P=9, k=1), 1010 (P=10, k=2), 123123 (P=123, k=3).
//
// Key insight: repeating P exactly m times gives:
//
//   N = P * (10^(km) - 1) / (10^k - 1)
//
// For m=2 this simplifies to P * (10^k + 1), e.g. k=2 → mult=101, so
// 12*101=1212, 99*101=9999.
//
// This algebraic structure lets us avoid iterating every ID in each range.

pub fn solve_part1(filename: &str) -> i64 {
    let ranges = parse_ranges(filename);
    let max_id = ranges.iter().map(|&(_, hi)| hi).max().unwrap_or(0);
    let mut total = 0i64;

    // For m=2: N = P * mult where mult = 10^k + 1.
    // P is a k-digit number, so p_min = 10^(k-1), p_max = 10^k - 1.
    //
    // For a range [lo, hi], the valid P values are:
    //   p_lo = ceil(lo / mult), clamped to [p_min, p_max]
    //   p_hi = floor(hi / mult), clamped to [p_min, p_max]
    //
    // Example: range [95, 115], k=1, mult=11
    //   p_lo = ceil(95/11) = 9, p_hi = floor(115/11) = 10 → clamped to 9
    //   → only P=9, N=99 ✓
    //
    // Example: range [998, 1012], k=2, mult=101
    //   p_lo = ceil(998/101) = 10, p_hi = floor(1012/101) = 10
    //   → only P=10, N=1010 ✓
    //
    // The sum of all N = P*mult for P in [p_lo..p_hi] is an arithmetic series:
    //   mult * (p_lo + p_lo+1 + ... + p_hi) = mult * count * (p_lo + p_hi) / 2
    for k in 1u32.. {
        let pow10k = 10_i64.pow(k);
        let p_min = if k == 1 { 1 } else { pow10k / 10 };
        let p_max = pow10k - 1;
        let mult = pow10k + 1;

        if p_min * mult > max_id {
            break;
        }

        for &(lo, hi) in &ranges {
            let p_lo = ((lo + mult - 1) / mult).max(p_min);
            let p_hi = (hi / mult).min(p_max);
            if p_lo <= p_hi {
                let count = p_hi - p_lo + 1;
                let sum_p = count * (p_lo + p_hi) / 2;
                total += sum_p * mult;
            }
        }
    }

    total
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

// Sum of all N = P * mult in [lo, hi], where mult = (10^l - 1)/(10^k - 1)
// and P is a k-digit number (no leading zeros).
fn arithmetic_sum(k: u32, l: u32, lo: i64, hi: i64) -> i64 {
    let pow10k = 10_i64.pow(k);
    let mult = (10_i64.pow(l) - 1) / (pow10k - 1);
    let p_min = if k == 1 { 1 } else { pow10k / 10 };
    let p_max = pow10k - 1;
    let p_lo = ((lo + mult - 1) / mult).max(p_min);
    let p_hi = (hi / mult).min(p_max);
    if p_lo > p_hi { return 0; }
    let count = p_hi - p_lo + 1;
    mult * count * (p_lo + p_hi) / 2
}

pub fn solve_part2(filename: &str) -> i64 {
    let ranges = parse_ranges(filename);
    let max_id = ranges.iter().map(|&(_, hi)| hi).max().unwrap_or(0);
    if max_id == 0 { return 0; }
    let max_l = max_id.ilog10() + 1;

    // For each digit length L, the invalid numbers are those with period k < L
    // (where k | L). Different (k, m) pairs with k*m = L can produce the same
    // number — e.g. 1111 = 11*101 (k=2) and 1*1111 (k=1, m=4). To sum each
    // invalid number exactly once, we use inclusion-exclusion over the proper
    // divisors of L.
    //
    // A number periodic with periods k₁ and k₂ has period gcd(k₁, k₂), so
    // intersections collapse to arithmetic_sum(gcd, l, ...). By
    // inclusion-exclusion:
    //
    //   sum(periodic) = Σ_{∅≠S ⊆ divs(L)} (-1)^(|S|+1) * arithmetic_sum(gcd(S), L, ...)
    //
    // Example for L=6, divs={1,2,3}: gcds of all pairs/triples reduce to 1,
    // so the formula gives arithmetic_sum(2,6) + arithmetic_sum(3,6) - arithmetic_sum(1,6).

    // Precompute proper divisors (periods < L that divide L) for each digit length.
    let divisors: Vec<Vec<u32>> = (0..=max_l)
        .map(|l| (1..l).filter(|&d| l % d == 0).collect())
        .collect();

    ranges
        .iter()
        .map(|&(lo, hi)| {
            let mut total = 0i64;
            for l in 2..=max_l {
                let pow10l = 10_i64.pow(l);
                // Clamp [lo, hi] to L-digit numbers.
                let lo_l = lo.max(pow10l / 10);
                let hi_l = hi.min(pow10l - 1);
                if lo_l > hi_l { continue; }

                let divs = &divisors[l as usize];
                let n = divs.len();
                for mask in 1u32..(1 << n) {
                    let sign = if mask.count_ones() % 2 == 1 { 1i64 } else { -1i64 };
                    let g = (0..n)
                        .filter(|&i| mask & (1 << i) != 0)
                        .map(|i| divs[i])
                        .fold(0u32, |acc, d| if acc == 0 { d } else { gcd(acc, d) });
                    total += sign * arithmetic_sum(g, l, lo_l, hi_l);
                }
            }
            total
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
11-22,95-115,998-1012,1188511880-1188511890,222220-222224,\
1698522-1698528,446443-446449,38593856-38593862,565653-565659,\
824824821-824824827,2121212118-2121212124
";

    #[test]
    fn example() {
        let tmp = crate::testutil::TempFile::write(EXAMPLE);
        assert_eq!(solve_part1(tmp.path()), 1227775554);
        assert_eq!(solve_part2(tmp.path()), 4174379265);
    }
}
