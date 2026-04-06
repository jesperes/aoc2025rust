use std::fs;

// Parse the grid as a flat byte array with a fixed stride, padding rows with
// spaces so all rows are the same width. This avoids Vec<Vec<char>> and the
// per-character Unicode decoding; everything stays as ASCII bytes.
fn read_grid(filename: &str) -> (Vec<u8>, usize) {
    let data = fs::read_to_string(filename).unwrap();
    let lines: Vec<&str> = data.lines().collect();
    let stride = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    let mut grid = vec![b' '; lines.len() * stride];
    for (r, line) in lines.iter().enumerate() {
        grid[r * stride..r * stride + line.len()].copy_from_slice(line.as_bytes());
    }
    (grid, stride)
}

// Returns groups of column indices separated by all-space columns in data rows.
// Uses mem::take to avoid cloning.
fn problem_groups(grid: &[u8], stride: usize, ndata: usize) -> Vec<Vec<usize>> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for c in 0..stride {
        let is_sep = (0..ndata).all(|r| grid[r * stride + c] == b' ');
        if is_sep {
            if !current.is_empty() {
                groups.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        groups.push(current);
    }
    groups
}

fn operator(grid: &[u8], stride: usize, nrows: usize, cols: &[usize]) -> u8 {
    let op_row = (nrows - 1) * stride;
    cols.iter()
        .map(|&c| grid[op_row + c])
        .find(|&ch| ch == b'+' || ch == b'*')
        .unwrap_or(b'+')
}

fn apply(numbers: &[i128], op: u8) -> i128 {
    if op == b'+' { numbers.iter().sum() } else { numbers.iter().product() }
}

// Parse a number from a byte slice, ignoring spaces; returns None if all spaces.
fn parse_bytes(bytes: &[u8]) -> Option<i128> {
    let mut result = 0i128;
    let mut any = false;
    for &b in bytes {
        if b != b' ' {
            result = result * 10 + (b - b'0') as i128;
            any = true;
        }
    }
    if any { Some(result) } else { None }
}

pub fn solve_part1(filename: &str) -> i128 {
    let (grid, stride) = read_grid(filename);
    let nrows = grid.len() / stride;
    let ndata = nrows - 1;
    problem_groups(&grid, stride, ndata)
        .iter()
        .map(|cols| {
            let op = operator(&grid, stride, nrows, cols);
            let numbers: Vec<i128> = (0..ndata)
                .filter_map(|r| {
                    let row = r * stride;
                    parse_bytes(&cols.iter().map(|&c| grid[row + c]).collect::<Vec<_>>())
                })
                .collect();
            apply(&numbers, op)
        })
        .sum()
}

pub fn solve_part2(filename: &str) -> i128 {
    let (grid, stride) = read_grid(filename);
    let nrows = grid.len() / stride;
    let ndata = nrows - 1;
    problem_groups(&grid, stride, ndata)
        .iter()
        .map(|cols| {
            let op = operator(&grid, stride, nrows, cols);
            let numbers: Vec<i128> = cols
                .iter()
                .rev()
                .filter_map(|&c| {
                    let col_bytes: Vec<u8> = (0..ndata).map(|r| grid[r * stride + c]).collect();
                    parse_bytes(&col_bytes)
                })
                .collect();
            apply(&numbers, op)
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = "\
123 328  51 64
 45 64  387 23
  6 98  215 314
*   +   *   +
";

    #[test]
    fn example() {
        let path = "/tmp/day06_ex.txt";
        std::fs::write(path, EXAMPLE).unwrap();
        assert_eq!(solve_part1(path), 4277556);
        assert_eq!(solve_part2(path), 3263827);
    }
}
