use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve_part1(filename: &str) -> i64 {
    solve_generic(filename, 1)
}

pub fn solve_part2(filename: &str) -> i64 {
    solve_generic(filename, 2)
}

fn solve_generic(filename: &str, part: i32) -> i64 {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let mut output_joltage: i64 = 0;
    for line in reader.lines() {
        let s = line.unwrap();
        let num_batteries = if part == 1 { 2 } else { 12 };
        let j = find_max_joltage(&s, num_batteries);
        output_joltage += j;
    }

    output_joltage
}

fn find_max_joltage(line: &String, num_batteries: usize) -> i64 {
    let mut joltage = String::new();
    let mut i: usize = 0;
    let bytes = line.as_bytes();

    loop {
        if joltage.len() >= num_batteries {
            break;
        }

        let mut max: u8 = 0;
        let reserved = num_batteries - joltage.len() - 1;

        for j in i..(line.len() - reserved) {
            if bytes[j] > max {
                max = bytes[j];
                i = j + 1;
            }
        }

        joltage.push(max as char);
    }

    joltage.parse().unwrap()
}
