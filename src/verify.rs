use std::fs;
use std::path::Path;

/// Returns the expected answers (part1, part2) for a given day, fetching and
/// caching them from the AoC puzzle page if not already stored.
/// Returns None for a part if the puzzle hasn't been solved yet.
pub fn expected_answers(day: u32) -> (Option<String>, Option<String>) {
    let path = format!("inputs/day{:02}.answers", day);
    if Path::new(&path).exists() {
        return parse_cache(&path);
    }

    let session = crate::input::read_session();
    let url = format!("https://adventofcode.com/2025/day/{}", day);
    let client = reqwest::blocking::Client::new();
    let html = client
        .get(&url)
        .header("Cookie", format!("session={}", session.trim()))
        .header("User-Agent", "github.com/aoc2025rust")
        .send()
        .unwrap_or_else(|e| panic!("failed to fetch puzzle page for day {}: {}", day, e))
        .text()
        .unwrap_or_else(|e| panic!("failed to read puzzle page for day {}: {}", day, e));

    let answers = parse_answers_from_html(&html);

    let cache = format!(
        "{}\n{}\n",
        answers.0.as_deref().unwrap_or(""),
        answers.1.as_deref().unwrap_or("")
    );
    fs::write(&path, cache)
        .unwrap_or_else(|e| panic!("failed to write answers cache {}: {}", path, e));

    answers
}

fn parse_answers_from_html(html: &str) -> (Option<String>, Option<String>) {
    let marker = "Your puzzle answer was <code>";
    let mut answers = vec![];
    let mut pos = 0;
    while let Some(offset) = html[pos..].find(marker) {
        let start = pos + offset + marker.len();
        if let Some(end) = html[start..].find("</code>") {
            answers.push(html[start..start + end].to_string());
            pos = start + end;
        } else {
            break;
        }
    }
    (answers.first().cloned(), answers.get(1).cloned())
}

fn parse_cache(path: &str) -> (Option<String>, Option<String>) {
    let content = fs::read_to_string(path).unwrap_or_default();
    let mut lines = content.lines();
    let p1 = lines.next().filter(|s| !s.is_empty()).map(String::from);
    let p2 = lines.next().filter(|s| !s.is_empty()).map(String::from);
    (p1, p2)
}
