use std::fs;

fn parse_line(line: &str) -> (u32, Vec<u32>, Vec<u64>, usize) {
    let bracket_end = line.find(']').unwrap();
    let pattern = &line[1..bracket_end];
    let n = pattern.len();
    let target = pattern.bytes().enumerate()
        .filter(|(_, b)| *b == b'#')
        .fold(0u32, |acc, (i, _)| acc | (1 << i));

    let rest = &line[bracket_end + 1..];
    let curly_start = rest.find('{').unwrap();
    let curly_end = rest.find('}').unwrap();
    let joltages: Vec<u64> = rest[curly_start + 1..curly_end]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    let mut buttons = Vec::new();
    let bytes = rest[..curly_start].as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'(' {
            i += 1;
            let j = i + rest[i..].find(')').unwrap();
            let mask = rest[i..j].split(',')
                .fold(0u32, |acc, s| acc | (1 << s.trim().parse::<usize>().unwrap()));
            buttons.push(mask);
            i = j + 1;
        } else {
            i += 1;
        }
    }

    (target, buttons, joltages, n)
}

// ── Part 1: GF(2) system ────────────────────────────────────────────────────

fn gauss_gf2(target: u32, buttons: &[u32], n: usize) -> Option<(Vec<u32>, Vec<usize>, Vec<usize>)> {
    let m = buttons.len();
    let mut mat: Vec<u32> = (0..n).map(|i| {
        buttons.iter().enumerate()
            .filter(|&(_, &btn)| (btn >> i) & 1 == 1)
            .fold(0u32, |acc, (j, _)| acc | (1 << j))
            | (((target >> i) & 1) << m)
    }).collect();

    let mut pivot_cols: Vec<usize> = Vec::new();
    let mut has_pivot = vec![false; m];
    let mut row = 0;

    for col in 0..m {
        if let Some(r) = (row..n).find(|&r| (mat[r] >> col) & 1 == 1) {
            mat.swap(row, r);
            pivot_cols.push(col);
            has_pivot[col] = true;
            let pivot_row = mat[row];
            for r2 in 0..n {
                if r2 != row && (mat[r2] >> col) & 1 == 1 { mat[r2] ^= pivot_row; }
            }
            row += 1;
        }
    }

    let rank = pivot_cols.len();
    if (rank..n).any(|r| (mat[r] >> m) & 1 == 1) { return None; }

    let free_vars: Vec<usize> = (0..m).filter(|&j| !has_pivot[j]).collect();
    Some((mat, pivot_cols, free_vars))
}

fn min_presses_gf2(mat: &[u32], pivot_cols: &[usize], free_vars: &[usize], m: usize) -> u64 {
    let k = free_vars.len();
    (0u32..1u32 << k).map(|free_mask| {
        let mut weight = free_mask.count_ones() as u64;
        for (ri, _) in pivot_cols.iter().enumerate() {
            let mut val = (mat[ri] >> m) & 1;
            for (fi, &fj) in free_vars.iter().enumerate() {
                if (mat[ri] >> fj) & 1 == 1 { val ^= (free_mask >> fi) & 1; }
            }
            weight += val as u64;
        }
        weight
    }).min().unwrap_or(u64::MAX)
}

pub fn solve_part1(filename: &str) -> u64 {
    fs::read_to_string(filename).unwrap().lines().map(|line| {
        let (target, buttons, _, n) = parse_line(line);
        let m = buttons.len();
        let (mat, pivot_cols, free_vars) = gauss_gf2(target, &buttons, n).unwrap();
        min_presses_gf2(&mat, &pivot_cols, &free_vars, m)
    }).sum()
}

// ── Part 2: integer LP via branch and bound ──────────────────────────────────
//
// Minimize  sum(x)   subject to  Ax = b,  x >= 0,  x integer.
// A[i][j] = 1 iff button j affects counter i;  b[i] = joltage target.
//
// LP relaxation solved with two-phase simplex; branch on first fractional var.

fn do_pivot(tab: &mut [f64], w: usize, nrows: usize, leave: usize, enter: usize) {
    let pv = tab[leave * w + enter];
    for j in 0..w { tab[leave * w + j] /= pv; }
    for i in 0..nrows {
        if i != leave {
            let f = tab[i * w + enter];
            if f.abs() > 1e-14 {
                for j in 0..w { let v = tab[leave * w + j]; tab[i * w + j] -= f * v; }
            }
        }
    }
}

// Solve LP: min sum(x[0..n_obj]) s.t. Ax = b, x >= 0.
// Two-phase simplex with Bland's rule in both phases.
// Phase 1 artificials are excluded from phase 2 to prevent re-entry.
// Returns Some((optimal, x)) or None if infeasible.
fn lp_solve(a: &[Vec<f64>], b: &[f64], n_obj: usize) -> Option<(f64, Vec<f64>)> {
    let n_eq = a.len();
    let n_var = a[0].len();
    // Columns: x[0..n_var], artificials art[n_var..n_var+n_eq], RHS at w-1
    let w = n_var + n_eq + 1;

    // Negate rows with negative RHS so initial BFS (artificials = b) is non-negative.
    let mut a_m = a.to_vec();
    let mut b_m = b.to_vec();
    for i in 0..n_eq {
        if b_m[i] < -1e-9 {
            b_m[i] = -b_m[i];
            for j in 0..n_var { a_m[i][j] = -a_m[i][j]; }
        }
    }

    // Build tableau: n_eq constraint rows + 1 objective row.
    let mut tab = vec![0.0f64; (n_eq + 1) * w];
    for i in 0..n_eq {
        for j in 0..n_var { tab[i*w + j] = a_m[i][j]; }
        tab[i*w + n_var + i] = 1.0;   // artificial column i
        tab[i*w + w - 1] = b_m[i];
    }
    let mut basis: Vec<usize> = (0..n_eq).map(|i| n_var + i).collect();

    // ── Phase 1: minimize sum(artificials) ──────────────────────────────────
    // Initial reduced costs with all artificials basic:
    //   c̄_j = 0 - sum_i a_m[i][j]  for original vars (c_j=0 in phase 1)
    //   c̄_art_i = 0                  for artificials (c=1, z=1, cancel)
    // RHS stores -(current phase-1 objective) = -sum(b_m)
    for j in 0..n_var {
        tab[n_eq*w + j] = -(0..n_eq).map(|i| a_m[i][j]).sum::<f64>();
    }
    tab[n_eq*w + w - 1] = -b_m.iter().sum::<f64>();

    simplex_bland(&mut tab, &mut basis, w, n_eq, n_var + n_eq, 2_000_000);

    // Infeasible if phase-1 objective > 0 (RHS < -eps → obj > eps)
    if tab[n_eq*w + w - 1] < -1e-6 { return None; }
    for i in 0..n_eq {
        if basis[i] >= n_var && tab[i*w + w - 1] > 1e-6 { return None; }
    }

    // ── Pivot out degenerate artificials still basic at 0 ───────────────────
    // Prevents them from re-entering in phase 2.
    for i in 0..n_eq {
        if basis[i] >= n_var {
            // Find any original variable with non-zero entry in this row
            if let Some(enter) = (0..n_var).find(|&j| tab[i*w + j].abs() > 1e-9) {
                basis[i] = enter;
                do_pivot(&mut tab, w, n_eq + 1, i, enter);
            }
            // No pivot available → redundant row; leave artificial as placeholder
        }
    }

    // ── Phase 2: minimize sum(x[0..n_obj]), excluding artificial columns ─────
    // Recompute objective row from current basis state.
    for j in 0..w { tab[n_eq*w + j] = 0.0; }
    for j in 0..n_obj { tab[n_eq*w + j] = 1.0; }  // c_j = 1 for j < n_obj
    // Subtract basis pricing: c̄_j = c_j - sum_i c_{basis[i]} * tab[i*w+j]
    for i in 0..n_eq {
        let bv = basis[i];
        let cbv = if bv < n_obj { 1.0f64 } else { 0.0 };
        if cbv.abs() > 1e-14 {
            for j in 0..w {
                let v = tab[i*w + j];
                tab[n_eq*w + j] -= cbv * v;
            }
        }
    }

    // Phase 2 only enters original variables (never artificials)
    simplex_bland(&mut tab, &mut basis, w, n_eq, n_var, 2_000_000);

    let opt = -tab[n_eq*w + w - 1];
    let mut x = vec![0.0f64; n_var];
    for i in 0..n_eq {
        if basis[i] < n_var { x[basis[i]] = tab[i*w + w - 1]; }
    }
    Some((opt, x))
}

fn simplex_bland(tab: &mut [f64], basis: &mut [usize], w: usize, n_eq: usize, n_enter: usize, max_iter: usize) {
    for _ in 0..max_iter {
        let enter = (0..n_enter).find(|&j| tab[n_eq*w + j] < -1e-9);
        let enter = match enter { Some(e) => e, None => break };
        let leave = (0..n_eq).filter(|&i| tab[i*w + enter] > 1e-9)
            .min_by(|&i, &j| {
                let ri = tab[i*w+w-1]/tab[i*w+enter];
                let rj = tab[j*w+w-1]/tab[j*w+enter];
                ri.partial_cmp(&rj).unwrap().then(basis[i].cmp(&basis[j]))
            });
        let leave = match leave { Some(l) => l, None => break };
        basis[leave] = enter;
        do_pivot(tab, w, n_eq + 1, leave, enter);
    }
}

// Branch and bound with proper inequality branching.
// lb[j]: lower bound for x[j] (x[j] >= lb[j]).
// ub[j]: optional upper bound for x[j] (x[j] <= ub[j]).
// Branching on fractional x[j]*=v uses x[j]>=ceil (update lb) and x[j]<=floor (update ub).
fn bnb(a: &[Vec<f64>], b: &[f64], lb: &[f64], ub: &[Option<f64>], best: &mut f64) -> f64 {
    let n = a.len();
    let m = a[0].len();

    // Check if any ub < lb (infeasible)
    for j in 0..m {
        if let Some(u) = ub[j] {
            if u < lb[j] - 1e-9 { return *best; }
        }
    }

    let lb_sum: f64 = lb.iter().sum();

    // Build augmented LP: original equality constraints + upper bound constraints.
    // For x[j] <= ub[j]: use shifted y[j] = x[j]-lb[j] >= 0, so y[j] <= ub[j]-lb[j].
    // Upper bound as equality: y[j] + slack_k = ub[j]-lb[j], slack_k >= 0.
    let ub_rows: Vec<(usize, f64)> = (0..m)
        .filter_map(|j| ub[j].map(|u| (j, u - lb[j])))
        .collect();
    let n_slack = ub_rows.len();

    // Adjust RHS: b_adj = b - A * lb
    let b_adj: Vec<f64> = (0..n).map(|i| {
        b[i] - (0..m).map(|j| a[i][j] * lb[j]).sum::<f64>()
    }).collect();

    // Build augmented matrix (n + n_slack rows, m + n_slack cols)
    let n_aug = n + n_slack;
    let m_aug = m + n_slack;
    let mut a_aug: Vec<Vec<f64>> = Vec::with_capacity(n_aug);
    for row in a.iter() {
        let mut r = row.to_vec();
        r.extend(vec![0.0; n_slack]);
        a_aug.push(r);
    }
    for (k, &(j, rhs)) in ub_rows.iter().enumerate() {
        let _ = rhs; // used in b_aug below
        let mut r = vec![0.0; m_aug];
        r[j] = 1.0;
        r[m + k] = 1.0; // slack
        a_aug.push(r);
    }
    let mut b_aug: Vec<f64> = b_adj.clone();
    for &(_, rhs) in &ub_rows { b_aug.push(rhs); }

    // Infeasible if any b_aug (for original rows) is sharply negative (checked by LP),
    // or any ub constraint RHS < 0 (ub < lb).
    for &(_, rhs) in &ub_rows {
        if rhs < -1e-9 { return *best; }
    }

    // LP: minimize sum(y[0..m]), where y[j] = x[j] - lb[j] >= 0.
    // n_obj = m (exclude slacks from objective)
    let (lp_y, x_aug) = match lp_solve(&a_aug, &b_aug, m) {
        Some(r) => r,
        None => return *best,
    };

    let total = lp_y + lb_sum;
    if total >= *best - 1e-6 { return *best; }

    // Check for fractional original variables (first m of x_aug)
    let frac = x_aug[..m].iter().enumerate()
        .find(|&(_, &v)| (v.max(0.0) - v.max(0.0).round()).abs() > 1e-5);

    match frac {
        None => {
            // All y[j] are integer; verify against original equality constraints.
            let y_int: Vec<f64> = x_aug[..m].iter().map(|&v| v.max(0.0).round()).collect();
            let feasible = (0..n).all(|i| {
                let lhs: f64 = (0..m).map(|j| a[i][j] * y_int[j]).sum();
                (lhs - b_adj[i]).abs() < 0.5
            });
            let int_sum = y_int.iter().sum::<f64>() + lb_sum;
            if feasible && int_sum < *best { *best = int_sum; }
            *best
        }
        Some((k, &vk)) => {
            let vk = vk.max(0.0);
            let floor_v = vk.floor();
            let ceil_v = vk.ceil();

            // Ceil branch: x[k] >= lb[k] + ceil_v  →  new lb[k] = lb[k] + ceil_v
            let mut new_lb = lb.to_vec();
            new_lb[k] += ceil_v;
            bnb(a, b, &new_lb, ub, best);

            // Floor branch: x[k] <= lb[k] + floor_v  →  new ub[k] = lb[k] + floor_v
            let mut new_ub = ub.to_vec();
            new_ub[k] = Some(match ub[k] {
                Some(prev) => prev.min(lb[k] + floor_v),
                None => lb[k] + floor_v,
            });
            bnb(a, b, lb, &new_ub, best);

            *best
        }
    }
}

pub fn solve_part2(filename: &str) -> u64 {
    fs::read_to_string(filename).unwrap().lines().map(|line| {
        let (_, buttons, joltages, n) = parse_line(line);
        let m = buttons.len();
        let a: Vec<Vec<f64>> = (0..n).map(|i| {
            buttons.iter().map(|&btn| ((btn >> i) & 1) as f64).collect()
        }).collect();
        let b: Vec<f64> = joltages.iter().map(|&x| x as f64).collect();
        let lb = vec![0.0f64; m];
        let ub = vec![None; m];
        let mut best = f64::INFINITY;
        bnb(&a, &b, &lb, &ub, &mut best);
        best.round() as u64
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}
";

    #[test]
    fn example() {
        let tmp = crate::testutil::TempFile::write(EXAMPLE);
        assert_eq!(solve_part1(tmp.path()), 7);
        assert_eq!(solve_part2(tmp.path()), 33);
    }

    #[test]
    fn debug_line_123() {
        let line = "[##..###] (0,1,2,3,6) (0,2) (0,2,4,5,6) (0,1,3,4) (0,2,3,4,5) (3,5,6) (4,6) (0,1,2,3,5,6) (2,3,5) {145,3,159,138,150,155,49}";
        let (_, buttons, joltages, n) = parse_line(line);
        let m = buttons.len();
        let a: Vec<Vec<f64>> = (0..n).map(|i| {
            buttons.iter().map(|&btn| ((btn >> i) & 1) as f64).collect()
        }).collect();
        let b: Vec<f64> = joltages.iter().map(|&x| x as f64).collect();
        let lb = vec![0.0f64; m];
        let ub = vec![None; m];
        let mut best = f64::INFINITY;
        bnb(&a, &b, &lb, &ub, &mut best);
        assert_eq!(best.round() as u64, 187);
    }
}
