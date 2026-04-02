use std::fs;

fn parse(filename: &str) -> Vec<(i64, i64)> {
    fs::read_to_string(filename)
        .unwrap()
        .lines()
        .map(|l| {
            let (x, y) = l.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

// Part 1: brute-force O(N²) over all pairs; area is inclusive on both ends.
pub fn solve_part1(filename: &str) -> i64 {
    let pts = parse(filename);
    let n = pts.len();
    let mut best = 0i64;
    for i in 0..n {
        for j in i + 1..n {
            let area = ((pts[i].0 - pts[j].0).abs() + 1) * ((pts[i].1 - pts[j].1).abs() + 1);
            if area > best { best = area; }
        }
    }
    best
}

// Part 2: the red tiles form a closed rectilinear polygon; green tiles are the
// polygon edges and interior. The rectangle must have red corners and contain
// only red/green tiles.
//
// Strategy:
//   1. Coordinate-compress the red tile x/y values.
//   2. Build an expanded grid (2N+1 × 2M+1) where odd indices = tile coords
//      and even indices = gaps. Mark polygon edges as boundary.
//   3. Flood-fill from the exterior to find all outside cells.
//   4. Build a 2D prefix sum over outside cells.
//   5. For each pair of red tiles, check (in O(1)) if their rectangle contains
//      zero outside cells, then track the maximum valid area.
pub fn solve_part2(filename: &str) -> i64 {
    let pts = parse(filename);
    let n = pts.len();

    // Coordinate compression
    let mut xs: Vec<i64> = pts.iter().map(|&(x, _)| x).collect();
    let mut ys: Vec<i64> = pts.iter().map(|&(_, y)| y).collect();
    xs.sort_unstable(); xs.dedup();
    ys.sort_unstable(); ys.dedup();
    let nx = xs.len();
    let ny = ys.len();

    // Expanded grid stored flat for cache locality; row-major: index = i*gh + j
    let gw = 2 * nx + 1;
    let gh = 2 * ny + 1;

    // Precompute expanded indices for every red tile (avoids binary search in hot loop)
    let ei: Vec<usize> = pts.iter().map(|&(x, _)| 2 * xs.binary_search(&x).unwrap() + 1).collect();
    let ej: Vec<usize> = pts.iter().map(|&(_, y)| 2 * ys.binary_search(&y).unwrap() + 1).collect();

    // Single state array with 1-cell padding on all sides.
    // Padded border = OUTSIDE (pre-set), so DFS never needs bounds checks
    // and can compute neighbors as p±pw/p±1 without division/modulo.
    // Values: 0 = unvisited interior, BOUNDARY = 1, OUTSIDE = 2.
    const BOUNDARY: u8 = 1;
    const OUTSIDE: u8 = 2;
    let pw = gw + 2;  // padded width
    let pdh = gh + 2; // padded height
    let mut state = vec![0u8; pw * pdh];

    // Mark padded border as OUTSIDE
    for i in 0..pw {
        state[i * pdh] = OUTSIDE;
        state[i * pdh + pdh - 1] = OUTSIDE;
    }
    for j in 0..pdh {
        state[j] = OUTSIDE;
        state[(pw - 1) * pdh + j] = OUTSIDE;
    }

    // Mark polygon boundary (padded coords = expanded coords + 1)
    for k in 0..n {
        let next = (k + 1) % n;
        let (i1, j1) = (ei[k] + 1, ej[k] + 1);
        let (i2, j2) = (ei[next] + 1, ej[next] + 1);
        if i1 == i2 {
            let (jlo, jhi) = (j1.min(j2), j1.max(j2));
            state[i1 * pdh + jlo..=i1 * pdh + jhi].fill(BOUNDARY);
        } else {
            let (ilo, ihi) = (i1.min(i2), i1.max(i2));
            for i in ilo..=ihi { state[i * pdh + j1] = BOUNDARY; }
        }
    }

    // DFS from original-grid border cells — no bounds checks needed.
    // Use u32 indices (max ~250K < 2³²) to halve stack memory vs usize.
    let mut stack: Vec<u32> = Vec::new();
    for i in 1..=gw {
        for &j in &[1usize, gh] {
            let p = i * pdh + j;
            if state[p] == 0 { state[p] = OUTSIDE; stack.push(p as u32); }
        }
    }
    for j in 2..gh {
        for &i in &[1usize, gw] {
            let p = i * pdh + j;
            if state[p] == 0 { state[p] = OUTSIDE; stack.push(p as u32); }
        }
    }
    while let Some(p) = stack.pop() {
        let p = p as usize;
        for q in [p - pdh, p + pdh, p - 1, p + 1] {
            if state[q] == 0 { state[q] = OUTSIDE; stack.push(q as u32); }
        }
    }

    // 2D prefix sum over original (unpadded) cells; row-major width = gh+1
    let psh = gh + 1;
    let mut psum = vec![0i32; (gw + 1) * psh];
    for i in 0..gw {
        for j in 0..gh {
            psum[(i + 1) * psh + (j + 1)] =
                (state[(i + 1) * pdh + (j + 1)] == OUTSIDE) as i32
                + psum[i * psh + (j + 1)]
                + psum[(i + 1) * psh + j]
                - psum[i * psh + j];
        }
    }

    // Sort tile indices by expanded x-index so the outer loop's psum row (ilo)
    // is always ei[ord[i]], letting us preload it into L1 cache for all inner-loop accesses.
    let mut ord: Vec<usize> = (0..n).collect();
    ord.sort_unstable_by_key(|&k| ei[k]);

    let mut best = 0i64;
    for ii in 0..n {
        let i = ord[ii];
        let (x1, y1) = pts[i];
        let (ei1, ej1) = (ei[i], ej[i]);
        // Preload the lower psum row (ei1 is the minimum x-index since ord is sorted)
        let row_lo = &psum[ei1 * psh..(ei1 + 1) * psh];
        for &j in &ord[ii + 1..] {
            let (x2, y2) = pts[j];
            if x1 == x2 || y1 == y2 { continue; }
            let ei2 = ei[j];
            let (jlo, jhi) = if ej1 <= ej[j] { (ej1, ej[j]) } else { (ej[j], ej1) };
            // query(ei1, jlo, ei2, jhi) — ei1 ≤ ei2 guaranteed by sort
            let val = psum[(ei2 + 1) * psh + jhi + 1]
                - psum[(ei2 + 1) * psh + jlo]
                - row_lo[jhi + 1]
                + row_lo[jlo];
            if val == 0 {
                let area = ((x1 - x2).abs() + 1) * ((y1 - y2).abs() + 1);
                if area > best { best = area; }
            }
        }
    }

    best
}
