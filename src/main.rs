mod day01;

use clap::{Parser, ValueEnum};
use std::fmt::Display;
use std::time::Instant;

#[derive(Debug, Clone, ValueEnum)]
enum Day {
    Day1,
}

#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    day: Day,

    filename: String,
}

fn benchmark<F, T>(name: &str, f: F)
where
    F: Fn() -> T,
    T: Display,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    println!(
        "{:4} -> {:12} took {:8} \u{03BC}s",
        name,
        result,
        duration.as_micros()
    );
}

fn main() {
    let cli = Cli::parse();
    let start = Instant::now();

    match cli.day {
        Day::Day1 => {
            benchmark("day 1 part 1", || day01::solve_part1(&cli.filename));
            benchmark("day 1 part 2", || day01::solve_part2(&cli.filename));
        }
    }

    let duration = start.elapsed();
    println!("Total time taken: {} ms", duration.as_millis());
}
