use std::{collections::BinaryHeap, fs};

// Coordinates are small (< 1024 in practice), so squared distances fit in i32
// (max 3 * 1000² = 3_000_000 << 2³¹). Using i32 with SoA layout gives the
// compiler the best shot at auto-vectorizing the inner distance loops.

struct Points {
    x: Vec<i32>,
    y: Vec<i32>,
    z: Vec<i32>,
}

impl Points {
    fn len(&self) -> usize { self.x.len() }

    fn dist2(&self, i: usize, j: usize) -> i64 {
        let dx = (self.x[i] - self.x[j]) as i64;
        let dy = (self.y[i] - self.y[j]) as i64;
        let dz = (self.z[i] - self.z[j]) as i64;
        dx*dx + dy*dy + dz*dz
    }
}

fn parse(filename: &str) -> Points {
    let data = fs::read_to_string(filename).unwrap();
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut z = Vec::new();
    for line in data.lines() {
        let mut it = line.split(',');
        x.push(it.next().unwrap().parse().unwrap());
        y.push(it.next().unwrap().parse().unwrap());
        z.push(it.next().unwrap().parse().unwrap());
    }
    Points { x, y, z }
}

struct UnionFind {
    parent: Vec<usize>,
    size: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        UnionFind { parent: (0..n).collect(), size: vec![1; n] }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]];
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra == rb { return; }
        if self.size[ra] < self.size[rb] {
            self.parent[ra] = rb; self.size[rb] += self.size[ra];
        } else {
            self.parent[rb] = ra; self.size[ra] += self.size[rb];
        }
    }
}

// Find the k globally closest pairs with a max-heap of size k.
//
// Sort points by x first. Then for each i, the inner j-loop only goes right
// (x[j] >= x[i]) and dx grows monotonically, so we can break as soon as
// dx² alone exceeds the current heap threshold — skipping all further j.
fn top3_circuit_sizes(pts: &Points, k: usize) -> i64 {
    let n = pts.len();

    let mut order: Vec<usize> = (0..n).collect();
    order.sort_unstable_by_key(|&i| pts.x[i]);

    let sx: Vec<i32> = order.iter().map(|&i| pts.x[i]).collect();
    let sy: Vec<i32> = order.iter().map(|&i| pts.y[i]).collect();
    let sz: Vec<i32> = order.iter().map(|&i| pts.z[i]).collect();

    let mut heap: BinaryHeap<(i64, usize, usize)> = BinaryHeap::with_capacity(k + 1);
    let mut thresh = i64::MAX;

    for i in 0..n {
        let xi = sx[i] as i64;
        let yi = sy[i] as i64;
        let zi = sz[i] as i64;
        let oi = order[i];

        for j in i + 1..n {
            let dx = sx[j] as i64 - xi;
            if dx * dx >= thresh { break; }

            let dy = sy[j] as i64 - yi;
            let dz = sz[j] as i64 - zi;
            let d = dx*dx + dy*dy + dz*dz;

            if heap.len() < k {
                heap.push((d, oi, order[j]));
                if heap.len() == k { thresh = heap.peek().unwrap().0; }
            } else if d < thresh {
                heap.pop();
                heap.push((d, oi, order[j]));
                thresh = heap.peek().unwrap().0;
            }
        }
    }

    let mut uf = UnionFind::new(n);
    for (_, i, j) in heap { uf.union(i, j); }

    let mut sizes: Vec<usize> = (0..n)
        .filter_map(|i| (uf.find(i) == i).then_some(uf.size[i]))
        .collect();
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    sizes[0] as i64 * sizes[1] as i64 * sizes[2] as i64
}

pub fn solve_part1(filename: &str) -> i64 {
    top3_circuit_sizes(&parse(filename), 1000)
}

// Prim's MST, O(N²). The max-weight MST edge is the last connection needed
// to unify all boxes. Maintain compact SoA arrays for remaining nodes so the
// inner loop has sequential memory access. Merge find-min and update-dist into
// a single pass so we only traverse the arrays once per MST node added.
pub fn solve_part2(filename: &str) -> i64 {
    let pts = parse(filename);
    let n = pts.len();

    // Compact SoA for remaining (not-yet-MST) nodes; swap_remove to evict.
    let mut rx: Vec<i32> = pts.x[1..].to_vec();
    let mut ry: Vec<i32> = pts.y[1..].to_vec();
    let mut rz: Vec<i32> = pts.z[1..].to_vec();
    let mut rorig: Vec<usize> = (1..n).collect();
    let mut rdist: Vec<i64> = (1..n).map(|v| pts.dist2(0, v)).collect();
    let mut rnear: Vec<usize> = vec![0; n - 1];

    let mut last_i = 0usize;
    let mut last_j = 0usize;
    let mut max_d = 0i64;

    // Seed: find initial minimum among distances from node 0.
    let mut cur_min = rdist.iter().enumerate().min_by_key(|&(_, &d)| d).unwrap().0;

    while !rorig.is_empty() {
        let min_pos = cur_min;
        let u      = rorig[min_pos];
        let ud     = rdist[min_pos];
        let u_near = rnear[min_pos];
        let xu     = rx[min_pos] as i64;
        let yu     = ry[min_pos] as i64;
        let zu     = rz[min_pos] as i64;

        // Evict.
        rorig.swap_remove(min_pos);
        rx.swap_remove(min_pos);
        ry.swap_remove(min_pos);
        rz.swap_remove(min_pos);
        rdist.swap_remove(min_pos);
        rnear.swap_remove(min_pos);

        if ud > max_d {
            max_d = ud;
            last_i = u_near;
            last_j = u;
        }

        // Update distances and find next minimum in a single pass.
        let mut next_d = i64::MAX;
        cur_min = 0;
        for k in 0..rx.len() {
            let dx = xu - rx[k] as i64;
            let dy = yu - ry[k] as i64;
            let dz = zu - rz[k] as i64;
            let d = dx*dx + dy*dy + dz*dz;
            if d < rdist[k] { rdist[k] = d; rnear[k] = u; }
            if rdist[k] < next_d { next_d = rdist[k]; cur_min = k; }
        }
    }

    pts.x[last_i] as i64 * pts.x[last_j] as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689
";

    #[test]
    fn example() {
        let tmp = crate::testutil::TempFile::write(EXAMPLE);
        assert_eq!(top3_circuit_sizes(&parse(tmp.path()), 10), 40);
        assert_eq!(solve_part2(tmp.path()), 25272);
    }
}
