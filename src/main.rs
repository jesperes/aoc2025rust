mod day01;
mod day02;
mod day03;
mod day04;
mod day05;
mod day06;
mod day07;
mod day08;
mod day09;
mod day10;
mod day11;
mod day12;
mod input;
mod verify;
#[cfg(test)]
mod testutil;

use clap::{Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(about = "Advent of Code 2025 solutions")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Run and benchmark solutions (default)
    Run {
        /// Run only this day (1-25)
        day: Option<u32>,
        /// Run only this part (1 or 2)
        part: Option<u32>,
        /// Number of benchmark iterations per solution
        #[arg(short, long, default_value_t = DEFAULT_RUNS)]
        runs: u32,
        /// Write benchmark results to this JSON file
        #[arg(short, long, default_value = "bench/rust.json")]
        json: String,
    },
    /// Print the puzzle input for a day, downloading it if needed
    Input {
        day: u32,
    },
    /// Print the cached answers for a day, downloading them if needed
    Answer {
        day: u32,
    },
    /// Print the puzzle description for a day, downloading it if needed
    Puzzle {
        day: u32,
    },
    /// Compare benchmark results from multiple languages
    Compare {
        /// Two or more bench JSON files to compare
        #[arg(required = true, num_args = 2..)]
        files: Vec<String>,
    },
}

const BENCH_TIMEOUT: Duration = Duration::from_secs(5);
const DEFAULT_RUNS: u32 = 100;

// Column widths (content between │ bars, including surrounding spaces):
//   col 1 (name):   │ {:<14} │  →  16
//   col 2 (answer): │ {:>20} │  →  22
//   col 3 (time):   │ {:>10} μs │  →  15  (1 + 10 + " μs " = 4)
//   col 4 (runs):   │ {:>3} runs │  →  10  (1 + 3 + " runs " = 6)
//   col 5 (check):  │ {}   │  →  5   (1 + 1 + 3)
const TOP_BORDER: &str    = "  ┌────────────────┬──────────────────────┬───────────────┬──────────┬─────┐";
const BOTTOM_BORDER: &str = "  └────────────────┴──────────────────────┴───────────────┴──────────┴─────┘";

#[derive(Serialize, Deserialize)]
struct BenchResult {
    answer: String,
    avg_micros: u64,
    runs: u32,
}

#[derive(Serialize)]
struct BenchEntry<'a> {
    name: &'a str,
    day: u32,
    part: u32,
    #[serde(flatten)]
    result: &'a BenchResult,
}

#[derive(Serialize)]
struct BenchOutput<'a> {
    language: &'static str,
    total_avg_micros: u64,
    solutions: Vec<BenchEntry<'a>>,
}

#[derive(Deserialize)]
struct BenchEntryOwned {
    name: String,
    day: u32,
    part: u32,
    #[serde(flatten)]
    result: BenchResult,
}

#[derive(Deserialize)]
struct BenchOutputOwned {
    language: String,
    #[allow(dead_code)]
    total_avg_micros: u64,
    solutions: Vec<BenchEntryOwned>,
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
            "│ {:<14} │ {:>20} │ {:>10} μs │ {:>3}      │     │",
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
        avg_micros: (total / runs).as_micros() as u64,
        runs,
    }
}

fn format_row(name: &str, result: &BenchResult, expected: Option<&str>) -> String {
    let check = match expected {
        Some(exp) if result.answer == exp => "\x1b[32m✓\x1b[0m",
        Some(_) => "\x1b[31m✗\x1b[0m",
        None => " ",
    };
    let time_color = if result.avg_micros < 1000 { "\x1b[32m" } else { "\x1b[31m" };
    format!(
        "│ {:<14} │ {:>20} │ {}{:>10}\x1b[0m μs │ {:>3} runs │ {}   │",
        name, result.answer, time_color, result.avg_micros, result.runs, check
    )
}

fn main() {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Run { day: None, part: None, runs: DEFAULT_RUNS, json: "bench/rust.json".to_string() }) {
        Command::Input { day } => {
            let path = input::ensure_input(day);
            print!("{}", std::fs::read_to_string(&path).unwrap());
        }
        Command::Answer { day } => {
            let (p1, p2) = verify::expected_answers(day);
            println!("part 1: {}", p1.as_deref().unwrap_or("(not yet solved)"));
            println!("part 2: {}", p2.as_deref().unwrap_or("(not yet solved)"));
        }
        Command::Puzzle { day } => {
            let path = input::ensure_puzzle(day);
            print!("{}", std::fs::read_to_string(&path).unwrap());
        }
        Command::Run { day: filter_day, part: filter_part, runs, json } => {
            run_solutions(filter_day, filter_part, runs, &json);
        }
        Command::Compare { files } => {
            compare_results(&files);
        }
    }
}

fn run_solutions(filter_day: Option<u32>, filter_part: Option<u32>, bench_runs: u32, json_path: &str) {
    let day01 = input::ensure_input(1);
    let day02 = input::ensure_input(2);
    let day03 = input::ensure_input(3);
    let day04 = input::ensure_input(4);
    let day05 = input::ensure_input(5);
    let day06 = input::ensure_input(6);
    let day07 = input::ensure_input(7);
    let day08 = input::ensure_input(8);
    let day09 = input::ensure_input(9);
    let day10 = input::ensure_input(10);
    let day11 = input::ensure_input(11);
    let day12 = input::ensure_input(12);

    let (a1p1, a1p2) = verify::expected_answers(1);
    let (a2p1, a2p2) = verify::expected_answers(2);
    let (a3p1, a3p2) = verify::expected_answers(3);
    let (a4p1, a4p2) = verify::expected_answers(4);
    let (a5p1, a5p2) = verify::expected_answers(5);
    let (a6p1, a6p2) = verify::expected_answers(6);
    let (a7p1, a7p2) = verify::expected_answers(7);
    let (a8p1, a8p2) = verify::expected_answers(8);
    let (a9p1, a9p2) = verify::expected_answers(9);
    let (a10p1, a10p2) = verify::expected_answers(10);
    let (a11p1, a11p2) = verify::expected_answers(11);
    let (a12p1, _) = verify::expected_answers(12);

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
        ("day 5 part 1", 5, 1, a5p1, { let p = day05.clone(); Box::new(move || day05::solve_part1(&p).to_string()) }),
        ("day 5 part 2", 5, 2, a5p2, { let p = day05.clone(); Box::new(move || day05::solve_part2(&p).to_string()) }),
        ("day 6 part 1", 6, 1, a6p1, { let p = day06.clone(); Box::new(move || day06::solve_part1(&p).to_string()) }),
        ("day 6 part 2", 6, 2, a6p2, { let p = day06.clone(); Box::new(move || day06::solve_part2(&p).to_string()) }),
        ("day 7 part 1", 7, 1, a7p1, { let p = day07.clone(); Box::new(move || day07::solve_part1(&p).to_string()) }),
        ("day 7 part 2", 7, 2, a7p2, { let p = day07.clone(); Box::new(move || day07::solve_part2(&p).to_string()) }),
        ("day 8 part 1", 8, 1, a8p1, { let p = day08.clone(); Box::new(move || day08::solve_part1(&p).to_string()) }),
        ("day 8 part 2", 8, 2, a8p2, { let p = day08.clone(); Box::new(move || day08::solve_part2(&p).to_string()) }),
        ("day 9 part 1", 9, 1, a9p1, { let p = day09.clone(); Box::new(move || day09::solve_part1(&p).to_string()) }),
        ("day 9 part 2", 9, 2, a9p2, { let p = day09.clone(); Box::new(move || day09::solve_part2(&p).to_string()) }),
        ("day 10 part 1", 10, 1, a10p1, { let p = day10.clone(); Box::new(move || day10::solve_part1(&p).to_string()) }),
        ("day 10 part 2", 10, 2, a10p2, { let p = day10.clone(); Box::new(move || day10::solve_part2(&p).to_string()) }),
        ("day 11 part 1", 11, 1, a11p1, { let p = day11.clone(); Box::new(move || day11::solve_part1(&p).to_string()) }),
        ("day 11 part 2", 11, 2, a11p2, { let p = day11.clone(); Box::new(move || day11::solve_part2(&p).to_string()) }),
        ("day 12 part 1", 12, 1, a12p1, { let p = day12.clone(); Box::new(move || day12::solve_part1(&p).to_string()) }),
    ];

    let solutions: Vec<_> = all_solutions
        .into_iter()
        .filter(|(_, day, part, _, _)| {
            filter_day.is_none_or(|d| d == *day) && filter_part.is_none_or(|p| p == *part)
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
                "│ {:<14} │ {:>20} │ {:>10}    │ {:>3}      │     │",
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

    let mut rows: Vec<String> = Vec::with_capacity(solutions.len());
    let mut bench_results: Vec<(&str, u32, u32, BenchResult)> = Vec::with_capacity(solutions.len());
    let mut total_avg_micros: u64 = 0;

    for (i, (name, day, part, expected, f)) in solutions.iter().enumerate() {
        let result = run_benchmark(name, bench_runs, &bars[i], f.as_ref());
        total_avg_micros += result.avg_micros;
        let row = format_row(name, &result, expected.as_deref());
        rows.push(row.clone());

        bars[i].set_style(ProgressStyle::with_template("  {msg}").unwrap());
        bars[i].finish_with_message(row);

        if let Some(next) = bars.get(i + 1) {
            next.set_style(spinner_style.clone());
            next.enable_steady_tick(Duration::from_millis(80));
        }

        bench_results.push((name, *day, *part, result));
    }

    mp.clear().unwrap();

    for row in &rows {
        println!("  {}", row);
    }
    println!("{}", BOTTOM_BORDER);

    let avg_ms = total_avg_micros as f64 / 1000.0;
    let color = if avg_ms < 1.0 { "\x1b[32m" } else { "\x1b[31m" };
    println!("\nTotal time: {}{:.1} ms\x1b[0m", color, avg_ms);

    let output = BenchOutput {
        language: "rust",
        total_avg_micros,
        solutions: bench_results.iter().map(|(name, day, part, r)| BenchEntry {
            name,
            day: *day,
            part: *part,
            result: r,
        }).collect(),
    };
    let json = serde_json::to_string_pretty(&output).unwrap();
    if let Some(parent) = std::path::Path::new(json_path).parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(json_path, json + "\n").unwrap();
    println!("Benchmark results written to {json_path}");
}

fn compare_results(files: &[String]) {
    let outputs: Vec<BenchOutputOwned> = files.iter().map(|f| {
        let text = std::fs::read_to_string(f)
            .unwrap_or_else(|e| panic!("Cannot read {f}: {e}"));
        serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("Cannot parse {f}: {e}"))
    }).collect();

    let languages: Vec<&str> = outputs.iter().map(|o| o.language.as_str()).collect();

    // (day, part) -> language -> avg_micros
    let mut table: BTreeMap<(u32, u32), BTreeMap<&str, u64>> = BTreeMap::new();
    // preserve canonical name per (day, part)
    let mut names: BTreeMap<(u32, u32), String> = BTreeMap::new();

    for output in &outputs {
        for entry in &output.solutions {
            let key = (entry.day, entry.part);
            names.entry(key).or_insert_with(|| entry.name.clone());
            table.entry(key).or_default().insert(output.language.as_str(), entry.result.avg_micros);
        }
    }

    // Column widths: name=16, each language=14 (right-aligned micros + ratio)
    let lang_col = 16usize;
    let name_col = 16usize;

    // Header
    let top: String = {
        let mut s = format!("  ┌{:─<name_col$}┬", "─".repeat(name_col));
        s.push_str(&languages.iter().map(|_| "─".repeat(lang_col)).collect::<Vec<_>>().join("┬"));
        s.push('┐');
        s
    };
    let sep: String = {
        let mut s = format!("  ├{:─<name_col$}┼", "─".repeat(name_col));
        s.push_str(&languages.iter().map(|_| "─".repeat(lang_col)).collect::<Vec<_>>().join("┼"));
        s.push('┤');
        s
    };
    let bot: String = {
        let mut s = format!("  └{:─<name_col$}┴", "─".repeat(name_col));
        s.push_str(&languages.iter().map(|_| "─".repeat(lang_col)).collect::<Vec<_>>().join("┴"));
        s.push('┘');
        s
    };

    println!("{top}");
    // Language header row
    let header_row = {
        let mut s = format!("  │ {:<width$}│", "solution", width = name_col - 1);
        for lang in &languages {
            s.push_str(&format!(" {:>width$}│", lang, width = lang_col - 1));
        }
        s
    };
    println!("{header_row}");
    println!("{sep}");

    // Totals per language for summary row — only puzzles where all languages have an entry
    let mut lang_totals: BTreeMap<&str, u64> = languages.iter().map(|&l| (l, 0u64)).collect();

    for (key, lang_map) in &table {
        let name = &names[key];
        let all_present = languages.iter().all(|&l| lang_map.contains_key(l));
        let fastest = lang_map.values().copied().min().unwrap_or(1).max(1);

        let mut row = format!("  │ {:<width$}│", name, width = name_col - 1);
        for &lang in &languages {
            if let Some(&micros) = lang_map.get(lang) {
                let ratio = micros as f64 / fastest as f64;
                let cell = format!("{micros} μs {ratio:.1}x");
                let color = if ratio < 1.5 { "\x1b[32m" } else if ratio < 3.0 { "\x1b[33m" } else { "\x1b[31m" };
                // right-align within lang_col-1 chars, then colorize
                row.push_str(&format!(" {color}{:>width$}\x1b[0m│", cell, width = lang_col - 1));
                if all_present {
                    *lang_totals.get_mut(lang).unwrap() += micros;
                }
            } else {
                row.push_str(&format!(" {:>width$}│", "-", width = lang_col - 1));
            }
        }
        println!("{row}");
    }

    // Totals row
    println!("{sep}");
    let total_fastest = lang_totals.values().copied().min().unwrap_or(1).max(1);
    let mut total_row = format!("  │ {:<width$}│", "total", width = name_col - 1);
    for &lang in &languages {
        let micros = lang_totals[lang];
        let ratio = micros as f64 / total_fastest as f64;
        let ms = micros as f64 / 1000.0;
        let cell = format!("{ms:.1} ms {ratio:.1}x");
        let color = if ratio < 1.5 { "\x1b[32m" } else if ratio < 3.0 { "\x1b[33m" } else { "\x1b[31m" };
        total_row.push_str(&format!(" {color}{:>width$}\x1b[0m│", cell, width = lang_col - 1));
    }
    println!("{total_row}");
    println!("{bot}");
}
