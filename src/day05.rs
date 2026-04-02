use std::fs;

fn parse(filename: &str) -> (Vec<(u64, u64)>, Vec<u64>) {
    let data = fs::read_to_string(filename).unwrap();
    let mut ranges = Vec::new();
    let mut ids = Vec::new();
    let mut in_ids = false;

    for line in data.lines() {
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

// Part 1: sort ranges by lo, build a prefix-max of hi, then for each ID
// binary-search for the insertion point and check the prefix max. O((N+M) log M).
// (A simple last-range check fails when an earlier broader range covers the ID.)
pub fn solve_part1(filename: &str) -> u64 {
    let (mut ranges, ids) = parse(filename);
    ranges.sort_unstable();
    let max_hi: Vec<u64> = {
        let mut m = 0u64;
        ranges.iter().map(|&(_, hi)| { m = m.max(hi); m }).collect()
    };
    let mut count = 0u64;
    for id in ids {
        let pos = ranges.partition_point(|&(lo, _)| lo <= id);
        if pos > 0 && max_hi[pos - 1] >= id {
            count += 1;
        }
    }
    count
}

// Part 2: sort and merge overlapping/adjacent ranges; sum their sizes.
pub fn solve_part2(filename: &str) -> u64 {
    let (mut ranges, _) = parse(filename);
    ranges.sort_unstable();

    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (lo, hi) in ranges {
        if let Some(last) = merged.last_mut()
            && lo <= last.1 + 1 {
            last.1 = last.1.max(hi);
            continue;
        }
        merged.push((lo, hi));
    }

    merged.iter().map(|(lo, hi)| hi - lo + 1).sum()
}
