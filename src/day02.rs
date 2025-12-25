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

fn is_invalid_id(s: &Vec<char>, w: usize, part: i32) -> bool {
    if w == 0 {
        return false;
    }
    if s.len() % w != 0 {
        // ignore widths which would yield a remainder
        return false;
    }

    let first = &s[0..w];
    let mut num_repeats = 0;

    for chunk in s.chunks_exact(w) {
        // println!("Chunk {:?}", chunk);
        if chunk == first {
            num_repeats += 1;
        } else {
            return false;
        }
    }

    if (part == 1 && num_repeats == 2) || (part == 2 && num_repeats >= 2) {
        return true;
    }

    return false;
}

fn solve_generic(filename: &str, part: i32) -> i64 {
    if let Ok(f) = File::open(filename) {
        let reader = BufReader::new(f);
        let line = reader.lines().next().unwrap().unwrap();
        let mut sum_invalid_ids: i64 = 0;

        // iterate over all ranges
        for range in line.split(',') {
            let r: Vec<_> = range.split("-").collect();
            let a = r[0];
            let b = r[1];
            let lower = a.parse::<i64>().unwrap();
            let upper = b.parse::<i64>().unwrap();

            // iterate over each range
            for id in lower..=upper {
                let s: Vec<char> = id.to_string().chars().collect();
                // part 1
                if part == 1 {
                    if is_invalid_id(&s, s.len() / 2, part) {
                        // println!("{} is invalid", id);
                        sum_invalid_ids += id;
                        break;
                    }
                } else {
                    // part 2
                    for w in 1..=(s.len() / 2) {
                        if is_invalid_id(&s, w, part) {
                            // println!("{} is invalid", id);
                            sum_invalid_ids += id;
                            break;
                        }
                    }
                }
            }
        }

        return sum_invalid_ids;
    } else {
        panic!();
    }
}
