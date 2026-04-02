mod day01;
mod day02;
mod day03;
mod day04;
mod input;
mod verify;

use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::fmt::Display;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(about = "Advent of Code 2025 solutions")]
struct Cli {
    /// Run only this day (1-25)
    day: Option<u32>,
    /// Run only this part (1 or 2)
    part: Option<u32>,
    /// Number of benchmark iterations per solution
    #[arg(short, long, default_value_t = 10)]
    runs: u32,
}

const BENCH_TIMEOUT: Duration = Duration::from_secs(5);

// Column widths (content between │ bars, including surrounding spaces):
//   col 1 (name):   │ {:<14} │  →  16
//   col 2 (answer): │ {:>20} │  →  22
//   col 3 (time):   │ {:>10} μs │  →  15  (1 + 10 + " μs " = 4)
//   col 4 (runs):   │ {:>2} runs │  →  9   (1 + 2 + " runs " = 6)
//   col 5 (check):  │ {} │  →  3   (1 + 1 + 1)
const TOP_BORDER: &str    = "  ┌────────────────┬──────────────────────┬───────────────┬─────────┬───┐";
const BOTTOM_BORDER: &str = "  └────────────────┴──────────────────────┴───────────────┴─────────┴───┘";

struct BenchResult {
    answer: String,
    avg_micros: u128,
    runs: u32,
}

fn run_benchmark<F, T>(name: &str, max_runs: u32, pb: &ProgressBar, f: F) -> BenchResult
where
    F: Fn() -> T,
    T: Display,
{
    let mut total = Duration::ZERO;
    let mut result = None;
    let mut runs = 0;
    for _ in 0..max_runs {
        pb.set_message(format!(
            "│ {:<14} │ {:>20} │ {:>10} μs │ {:>2}      │   │",
            name,
            "…",
            format!("{}/{}", runs + 1, max_runs),
            "…",
        ));
        let start = Instant::now();
        result = Some(f());
        total += start.elapsed();
        runs += 1;
        if total >= BENCH_TIMEOUT {
            break;
        }
    }
    BenchResult {
        answer: result.unwrap().to_string(),
        avg_micros: (total / runs).as_micros(),
        runs,
    }
}

fn format_row(name: &str, result: &BenchResult, expected: Option<&str>) -> String {
    let check = match expected {
        Some(exp) if result.answer == exp => "\x1b[32m✓\x1b[0m",
        Some(_) => "\x1b[31m✗\x1b[0m",
        None => " ",
    };
    format!(
        "│ {:<14} │ {:>20} │ {:>10} μs │ {:>2} runs │ {} │",
        name, result.answer, result.avg_micros, result.runs, check
    )
}

fn main() {
    let cli = Cli::parse();

    let day01 = input::ensure_input(1);
    let day02 = input::ensure_input(2);
    let day03 = input::ensure_input(3);
    let day04 = input::ensure_input(4);

    let (a1p1, a1p2) = verify::expected_answers(1);
    let (a2p1, a2p2) = verify::expected_answers(2);
    let (a3p1, a3p2) = verify::expected_answers(3);
    let (a4p1, a4p2) = verify::expected_answers(4);

    type Solution = (&'static str, u32, u32, Option<String>, Box<dyn Fn() -> String>);
    let all_solutions: Vec<Solution> = vec![
        ("day 1 part 1", 1, 1, a1p1, { let p = day01.clone(); Box::new(move || day01::solve_part1(&p).to_string()) }),
        ("day 1 part 2", 1, 2, a1p2, { let p = day01.clone(); Box::new(move || day01::solve_part2(&p).to_string()) }),
        ("day 2 part 1", 2, 1, a2p1, { let p = day02.clone(); Box::new(move || day02::solve_part1(&p).to_string()) }),
        ("day 2 part 2", 2, 2, a2p2, { let p = day02.clone(); Box::new(move || day02::solve_part2(&p).to_string()) }),
        ("day 3 part 1", 3, 1, a3p1, { let p = day03.clone(); Box::new(move || day03::solve_part1(&p).to_string()) }),
        ("day 3 part 2", 3, 2, a3p2, { let p = day03.clone(); Box::new(move || day03::solve_part2(&p).to_string()) }),
        ("day 4 part 1", 4, 1, a4p1, { let p = day04.clone(); Box::new(move || day04::solve_part1(&p).to_string()) }),
        ("day 4 part 2", 4, 2, a4p2, { let p = day04.clone(); Box::new(move || day04::solve_part2(&p).to_string()) }),
    ];

    let solutions: Vec<_> = all_solutions
        .into_iter()
        .filter(|(_, day, part, _, _)| {
            cli.day.map_or(true, |d| d == *day) && cli.part.map_or(true, |p| p == *part)
        })
        .collect();

    let spinner_style = ProgressStyle::with_template("{spinner:.cyan} {msg}")
        .unwrap()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]);
    let pending_style = ProgressStyle::with_template("  {msg}").unwrap();

    println!("{}", TOP_BORDER);

    let mp = MultiProgress::new();
    let bars: Vec<ProgressBar> = solutions
        .iter()
        .enumerate()
        .map(|(i, (name, _, _, _, _))| {
            let pb = mp.add(ProgressBar::new_spinner());
            let pending = format!(
                "│ {:<14} │ {:>20} │ {:>10}    │ {:>2}      │   │",
                name, "…", "…", "…"
            );
            if i == 0 {
                pb.set_style(spinner_style.clone());
                pb.enable_steady_tick(Duration::from_millis(80));
            } else {
                pb.set_style(pending_style.clone());
            }
            pb.set_message(pending);
            pb
        })
        .collect();

    let bottom_bar = mp.add(ProgressBar::new_spinner());
    bottom_bar.set_style(ProgressStyle::with_template("{msg}").unwrap());
    bottom_bar.finish_with_message(BOTTOM_BORDER);

    let start = Instant::now();
    let mut results: Vec<String> = Vec::with_capacity(solutions.len());

    for (i, (name, _, _, expected, f)) in solutions.iter().enumerate() {
        let result = run_benchmark(name, cli.runs, &bars[i], f.as_ref());
        let row = format_row(name, &result, expected.as_deref());
        results.push(row.clone());

        bars[i].set_style(ProgressStyle::with_template("  {msg}").unwrap());
        bars[i].finish_with_message(row);

        if let Some(next) = bars.get(i + 1) {
            next.set_style(spinner_style.clone());
            next.enable_steady_tick(Duration::from_millis(80));
        }
    }

    let duration = start.elapsed();
    mp.clear().unwrap();

    for row in &results {
        println!("  {}", row);
    }
    println!("{}", BOTTOM_BORDER);

    println!("\nTotal time: {} ms", duration.as_millis());
}
