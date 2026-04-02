use std::{
    fs::File,
    io::{BufRead, BufReader},
};

fn dir_to_delta(c: char) -> i32 {
    if c == 'L' { -1 } else { 1 }
}

pub fn solve_part1(filename: &str) -> i32 {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);

        let mut dial = 50;
        let mut zerocount = 0;
        let period = 100;

        for str in reader.lines().map_while(Result::ok) {
            let delta = dir_to_delta(str.chars().next().unwrap());
            let clicks = str[1..].parse::<i32>().unwrap();
            dial = (dial + period + clicks * delta) % period;
            if dial == 0 {
                zerocount += 1;
            }
        }
        return zerocount;
    }
    panic!()
}

pub fn solve_part2(filename: &str) -> i32 {
    if let Ok(file) = File::open(filename) {
        let reader = BufReader::new(file);

        let mut dial = 50;
        let mut zerocount = 0;
        let period = 100;

        for str in reader.lines().map_while(Result::ok) {
            let delta = dir_to_delta(str.chars().next().unwrap());
            let clicks = str[1..].parse::<i32>().unwrap();

            for _ in 0..clicks {
                dial = (dial + period + delta) % period;
                if dial == 0 {
                    zerocount += 1;
                }
            }
        }
        return zerocount;
    }
    panic!()
}
