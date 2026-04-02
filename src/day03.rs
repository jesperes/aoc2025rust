use std::fs;

// Both parts greedily pick the lexicographically largest digit sequence from
// each line. At step k, scan from the current position up to len-reserved,
// track the maximum byte, and advance i to just after where that max was found.
// Accumulate directly into i64 to avoid String building and parsing.
pub fn solve_part1(filename: &str) -> i64 { solve(filename, 2) }
pub fn solve_part2(filename: &str) -> i64 { solve(filename, 12) }

fn solve(filename: &str, num_batteries: usize) -> i64 {
    let data = fs::read_to_string(filename).unwrap();
    data.lines()
        .map(|l| find_max_joltage(l.as_bytes(), num_batteries))
        .sum()
}

#[allow(clippy::needless_range_loop, clippy::mut_range_bound)]
fn find_max_joltage(bytes: &[u8], num_batteries: usize) -> i64 {
    let mut result = 0i64;
    let mut i = 0;
    for step in 0..num_batteries {
        let reserved = num_batteries - step - 1;
        let mut max = 0u8;
        for j in i..bytes.len() - reserved {
            if bytes[j] > max {
                max = bytes[j];
                i = j + 1;
            }
        }
        result = result * 10 + (max - b'0') as i64;
    }
    result
}
