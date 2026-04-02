use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn parse(filename: &str) -> (Vec<(u64, u64)>, Vec<u64>) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut ranges = Vec::new();
    let mut ids = Vec::new();
    let mut in_ids = false;

    for line in reader.lines().map_while(Result::ok) {
        if line.is_empty() {
            in_ids = true;
            continue;
        }
        if in_ids {
            ids.push(line.parse::<u64>().unwrap());
        } else {
            let (a, b) = line.split_once('-').unwrap();
            ranges.push((a.parse::<u64>().unwrap(), b.parse::<u64>().unwrap()));
        }
    }
    (ranges, ids)
}

pub fn solve_part1(filename: &str) -> u64 {
    let (ranges, ids) = parse(filename);
    ids.iter()
        .filter(|&&id| ranges.iter().any(|&(lo, hi)| id >= lo && id <= hi))
        .count() as u64
}

pub fn solve_part2(filename: &str) -> u64 {
    let (mut ranges, _) = parse(filename);
    ranges.sort_unstable();

    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (lo, hi) in ranges {
        if let Some(last) = merged.last_mut() {
            if lo <= last.1 + 1 {
                last.1 = last.1.max(hi);
                continue;
            }
        }
        merged.push((lo, hi));
    }

    merged.iter().map(|(lo, hi)| hi - lo + 1).sum()
}
