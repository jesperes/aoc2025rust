use std::collections::HashSet;
use std::fs;

type Piece = Vec<(i32, i32)>;

fn d4_orientations(base: &[(i32, i32)]) -> Vec<Piece> {
    // All 8 elements of the dihedral group D4
    let transforms: [fn(i32, i32) -> (i32, i32); 8] = [
        |r, c| (r, c),
        |r, c| (c, -r),
        |r, c| (-r, -c),
        |r, c| (-c, r),
        |r, c| (r, -c),
        |r, c| (-c, -r),
        |r, c| (-r, c),
        |r, c| (c, r),
    ];
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for t in &transforms {
        let mut cells: Piece = base.iter().map(|&(r, c)| t(r, c)).collect();
        let min_r = cells.iter().map(|&(r, _)| r).min().unwrap();
        let min_c = cells.iter().map(|&(_, c)| c).min().unwrap();
        for (r, c) in &mut cells {
            *r -= min_r;
            *c -= min_c;
        }
        cells.sort_unstable();
        if seen.insert(cells.clone()) {
            result.push(cells);
        }
    }
    result
}

fn parse(filename: &str) -> (Vec<Vec<Piece>>, Vec<(usize, usize, Vec<usize>)>) {
    let text = fs::read_to_string(filename).unwrap();
    let mut lines = text.lines().peekable();
    let mut raw_shapes: Vec<Piece> = Vec::new();
    let mut regions = Vec::new();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((dims, counts_str)) = line.split_once(": ") {
            if let Some((ws, hs)) = dims.split_once('x') {
                if let (Ok(w), Ok(h)) = (ws.parse::<usize>(), hs.parse::<usize>()) {
                    let counts: Vec<usize> = counts_str
                        .split_whitespace()
                        .map(|s| s.parse().unwrap())
                        .collect();
                    regions.push((w, h, counts));
                }
            }
        } else if line.trim_end_matches(':').parse::<usize>().is_ok() {
            // Shape header: "0:", "1:", etc.
            let mut cells = Vec::new();
            for row in 0i32..3 {
                let row_line = lines.next().unwrap_or("");
                for (col, ch) in row_line.chars().enumerate() {
                    if ch == '#' {
                        cells.push((row, col as i32));
                    }
                }
            }
            raw_shapes.push(cells);
        }
    }

    let orientations = raw_shapes.iter().map(|s| d4_orientations(s)).collect();
    (orientations, regions)
}

/// Backtracking packer with symmetry breaking.
/// pieces[idx..] = remaining piece types to place (sorted by decreasing size).
/// min_cell[t] = minimum allowed flattened cell index for the next placement of type t.
/// remaining = total cells left to place, empty = total empty cells in grid.
fn pack(
    w: usize,
    h: usize,
    orientations: &[Vec<Piece>],
    pieces: &[usize],
    idx: usize,
    grid: &mut [bool],
    min_cell: &mut [usize],
    remaining: usize,
    empty: usize,
) -> bool {
    if idx == pieces.len() {
        return true;
    }
    if remaining > empty {
        return false;
    }

    let t = pieces[idx];
    let psize = orientations[t][0].len();
    let saved = min_cell[t];

    for orient in &orientations[t] {
        let max_r = orient.iter().map(|&(r, _)| r).max().unwrap() as usize;
        let max_c = orient.iter().map(|&(_, c)| c).max().unwrap() as usize;
        if max_r >= h || max_c >= w {
            continue;
        }

        // Skip anchors whose minimum cell index is below saved
        let min_ar = saved / w;

        for ar in min_ar..=(h - 1 - max_r) {
            for ac in 0..=(w - 1 - max_c) {
                // Compute minimum cell index for this placement
                let min_idx = orient
                    .iter()
                    .map(|&(dr, dc)| (ar as i32 + dr) as usize * w + (ac as i32 + dc) as usize)
                    .min()
                    .unwrap();
                if min_idx < saved {
                    continue;
                }

                // Check no overlap
                if orient.iter().any(|&(dr, dc)| {
                    grid[(ar as i32 + dr) as usize * w + (ac as i32 + dc) as usize]
                }) {
                    continue;
                }

                // Place
                for &(dr, dc) in orient.iter() {
                    grid[(ar as i32 + dr) as usize * w + (ac as i32 + dc) as usize] = true;
                }
                min_cell[t] = min_idx + 1;

                if pack(
                    w,
                    h,
                    orientations,
                    pieces,
                    idx + 1,
                    grid,
                    min_cell,
                    remaining - psize,
                    empty - psize,
                ) {
                    // Undo placement before returning true (caller doesn't need grid state)
                    for &(dr, dc) in orient.iter() {
                        grid[(ar as i32 + dr) as usize * w + (ac as i32 + dc) as usize] = false;
                    }
                    min_cell[t] = saved;
                    return true;
                }

                // Undo placement
                for &(dr, dc) in orient.iter() {
                    grid[(ar as i32 + dr) as usize * w + (ac as i32 + dc) as usize] = false;
                }
            }
        }
    }

    min_cell[t] = saved;
    false
}

pub fn solve_part1(filename: &str) -> usize {
    let (orientations, regions) = parse(filename);
    let sizes: Vec<usize> = orientations.iter().map(|o| o[0].len()).collect();

    let mut count = 0;
    for (w, h, region_counts) in &regions {
        let total_cells: usize = region_counts
            .iter()
            .zip(sizes.iter())
            .map(|(&c, &s)| c * s)
            .sum();

        if total_cells > w * h {
            continue; // Area infeasibility: fast reject
        }

        // Build flat piece list, sorted by decreasing piece size (hardest first)
        let mut pieces: Vec<usize> = (0..orientations.len())
            .flat_map(|t| std::iter::repeat(t).take(region_counts[t]))
            .collect();
        pieces.sort_by_key(|&t| std::cmp::Reverse(sizes[t]));

        let mut grid = vec![false; w * h];
        let mut min_cell = vec![0usize; orientations.len()];

        if pack(
            *w,
            *h,
            &orientations,
            &pieces,
            0,
            &mut grid,
            &mut min_cell,
            total_cells,
            w * h,
        ) {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example() {
        assert_eq!(solve_part1("/tmp/day12_ex.txt"), 2);
    }
}
