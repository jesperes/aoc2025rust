use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

fn parse(filename: &str) -> HashSet<(i32, i32)> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut rolls: HashSet<(i32, i32)> = HashSet::new();
    for (y, line) in reader.lines().enumerate() {
        for (x, c) in line.unwrap().chars().enumerate() {
            if c == '@' {
                rolls.insert((x as i32, y as i32));
            }
        }
    }
    rolls
}

fn is_removable(rolls: &HashSet<(i32, i32)>, roll: (i32, i32)) -> bool {
    let deltas = vec![
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    let mut num_reachable = 0;
    let (x, y) = roll;
    for (dx, dy) in &deltas {
        if rolls.contains(&(x + dx, y + dy)) {
            num_reachable += 1;
        }
    }

    return num_reachable < 4;
}

pub fn solve_part1(filename: &str) -> i32 {
    let rolls = parse(filename);
    let mut num_rolls = 0;

    for roll in &rolls {
        if is_removable(&rolls, *roll) {
            num_rolls += 1;
        }
    }

    num_rolls
}

pub fn solve_part2(filename: &str) -> i32 {
    let mut rolls = parse(filename);
    let total_rolls = rolls.len() as i32;

    loop {
        let mut removables: HashSet<(i32, i32)> = HashSet::new();

        for roll in &rolls {
            if is_removable(&rolls, *roll) {
                removables.insert(*roll);
            }
        }

        if removables.len() == 0 {
            break;
        }

        for roll in removables {
            rolls.remove(&roll);
        }
    }

    total_rolls - (rolls.len() as i32)
}
