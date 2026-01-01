mod day01;
mod day02;
mod day03;
mod day04;

use std::fmt::Display;
use std::time::Instant;

fn benchmark<F, T>(name: &str, f: F)
where
    F: Fn() -> T,
    T: Display,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!(
        "{:4} -> {:20} took {:8} \u{03BC}s",
        name,
        result,
        duration.as_micros()
    );
}

fn main() {
    let start = Instant::now();

    benchmark("day 1 part 1", || day01::solve_part1("inputs/day01.txt"));
    benchmark("day 1 part 2", || day01::solve_part2("inputs/day01.txt"));
    benchmark("day 2 part 1", || day02::solve_part1("inputs/day02.txt"));
    benchmark("day 2 part 2", || day02::solve_part2("inputs/day02.txt"));
    benchmark("day 3 part 1", || day03::solve_part1("inputs/day03.txt"));
    benchmark("day 3 part 2", || day03::solve_part2("inputs/day03.txt"));
    benchmark("day 4 part 1", || day04::solve_part1("inputs/day04.txt"));
    benchmark("day 4 part 1", || day04::solve_part2("inputs/day04.txt"));

    let duration = start.elapsed();
    println!("Total time taken: {} ms", duration.as_millis());
}
