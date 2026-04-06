use std::{collections::VecDeque, fs};

// Part 1: count rolls with fewer than 4 occupied neighbours.
// Part 2: naive approach rescans all remaining rolls each round — O(rounds * N).
// Instead, maintain a neighbour-count array and a removal queue. Removing a roll
// only affects its 8 neighbours' counts, so only they need rechecking. This gives
// O(N) total work regardless of how many rounds the peeling takes.
//
// Both parts use a flat padded grid (1-cell silent border on all sides) instead of
// HashSet<(i32,i32)>, so all neighbour lookups are direct array accesses with a
// precomputed signed offset — no hashing overhead.
// stride = cols + 2.  Interior cell (r, c) → index (r+1)*stride + (c+1).

fn parse(filename: &str) -> (Vec<bool>, usize) {
    let data = fs::read_to_string(filename).unwrap();
    let lines: Vec<&str> = data.lines().collect();
    let cols = lines[0].len();
    let stride = cols + 2;
    let total = (lines.len() + 2) * stride;
    let mut grid = vec![false; total];
    for (r, line) in lines.iter().enumerate() {
        for (c, ch) in line.chars().enumerate() {
            if ch == '@' {
                grid[(r + 1) * stride + (c + 1)] = true;
            }
        }
    }
    (grid, stride)
}

fn neighbor_offsets(stride: usize) -> [isize; 8] {
    let s = stride as isize;
    [-s - 1, -s, -s + 1, -1, 1, s - 1, s, s + 1]
}

pub fn solve_part1(filename: &str) -> i32 {
    let (grid, stride) = parse(filename);
    let offs = neighbor_offsets(stride);
    grid.iter()
        .enumerate()
        .filter(|&(i, &cell)| {
            cell && offs.iter().filter(|&&d| grid[(i as isize + d) as usize]).count() < 4
        })
        .count() as i32
}

pub fn solve_part2(filename: &str) -> i32 {
    let (mut grid, stride) = parse(filename);
    let offs = neighbor_offsets(stride);

    // Pre-compute neighbor counts for every cell.
    let mut ncnt: Vec<u8> = grid
        .iter()
        .enumerate()
        .map(|(i, &cell)| {
            if cell {
                offs.iter().filter(|&&d| grid[(i as isize + d) as usize]).count() as u8
            } else {
                0
            }
        })
        .collect();

    // Seed the queue with all initially removable rolls (neighbor count < 4).
    let mut queue: VecDeque<usize> = grid
        .iter()
        .enumerate()
        .filter_map(|(i, &cell)| (cell && ncnt[i] < 4).then_some(i))
        .collect();

    // Process removals. Removing a roll decrements the neighbor count of each
    // of its 8 neighbours; any that drop below 4 are added to the queue.
    let mut removed = 0i32;
    while let Some(i) = queue.pop_front() {
        if !grid[i] {
            continue; // duplicate in queue
        }
        grid[i] = false;
        removed += 1;
        for &d in &offs {
            let nb = (i as isize + d) as usize;
            if grid[nb] {
                ncnt[nb] -= 1;
                if ncnt[nb] < 4 {
                    queue.push_back(nb);
                }
            }
        }
    }

    removed
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.
";

    #[test]
    fn example() {
        let tmp = crate::testutil::TempFile::write(EXAMPLE);
        assert_eq!(solve_part1(tmp.path()), 13);
        assert_eq!(solve_part2(tmp.path()), 43);
    }
}
