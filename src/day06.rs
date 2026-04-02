use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn read_grid(filename: &str) -> Vec<Vec<char>> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map_while(Result::ok).collect();
    let max_len = lines.iter().map(|l| l.len()).max().unwrap_or(0);
    lines
        .iter()
        .map(|l| {
            let mut chars: Vec<char> = l.chars().collect();
            chars.resize(max_len, ' ');
            chars
        })
        .collect()
}

/// Groups of column indices, separated by all-space columns (ignoring operator row).
fn problem_groups(grid: &[Vec<char>]) -> Vec<Vec<usize>> {
    let ndata = grid.len() - 1;
    let ncols = grid[0].len();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut current: Vec<usize> = Vec::new();
    for c in 0..ncols {
        let is_sep = (0..ndata).all(|r| grid[r][c] == ' ');
        if is_sep {
            if !current.is_empty() {
                groups.push(current.clone());
                current.clear();
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

fn operator(grid: &[Vec<char>], cols: &[usize]) -> char {
    let op_row = &grid[grid.len() - 1];
    cols.iter()
        .map(|&c| op_row[c])
        .find(|&ch| ch == '+' || ch == '*')
        .unwrap_or('+')
}

fn apply(numbers: &[i128], op: char) -> i128 {
    if op == '+' {
        numbers.iter().sum()
    } else {
        numbers.iter().product()
    }
}

pub fn solve_part1(filename: &str) -> i128 {
    let grid = read_grid(filename);
    let ndata = grid.len() - 1;
    problem_groups(&grid)
        .iter()
        .map(|cols| {
            let op = operator(&grid, cols);
            let numbers: Vec<i128> = (0..ndata)
                .filter_map(|r| {
                    let s: String = cols.iter().map(|&c| grid[r][c]).filter(|ch| *ch != ' ').collect();
                    if s.is_empty() { None } else { Some(s.parse().unwrap()) }
                })
                .collect();
            apply(&numbers, op)
        })
        .sum()
}

pub fn solve_part2(filename: &str) -> i128 {
    let grid = read_grid(filename);
    let ndata = grid.len() - 1;
    problem_groups(&grid)
        .iter()
        .map(|cols| {
            let op = operator(&grid, cols);
            let numbers: Vec<i128> = cols
                .iter()
                .rev()
                .filter_map(|&c| {
                    let s: String = (0..ndata).map(|r| grid[r][c]).filter(|ch| *ch != ' ').collect();
                    if s.is_empty() { None } else { Some(s.parse().unwrap()) }
                })
                .collect();
            apply(&numbers, op)
        })
        .sum()
}
