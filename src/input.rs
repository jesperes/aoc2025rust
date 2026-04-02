use std::fs;
use std::path::Path;

/// Returns the path to the input file for the given day, downloading it if necessary.
/// The session cookie is read from the `AOC_SESSION` environment variable or the `.aoc_session` file.
pub fn ensure_input(day: u32) -> String {
    let path = format!("inputs/day{:02}.txt", day);
    if Path::new(&path).exists() {
        return path;
    }

    let session = read_session();
    fs::create_dir_all("inputs").expect("failed to create inputs directory");

    let url = format!("https://adventofcode.com/2025/day/{}/input", day);
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("Cookie", format!("session={}", session.trim()))
        .header("User-Agent", "github.com/aoc2025rust")
        .send()
        .unwrap_or_else(|e| panic!("failed to fetch input for day {}: {}", day, e));

    if !response.status().is_success() {
        panic!(
            "failed to fetch input for day {}: HTTP {}",
            day,
            response.status()
        );
    }

    let content = response
        .text()
        .unwrap_or_else(|e| panic!("failed to read response for day {}: {}", day, e));

    fs::write(&path, &content)
        .unwrap_or_else(|e| panic!("failed to write input file {}: {}", path, e));

    println!("Downloaded input for day {}", day);
    path
}

/// Returns the path to the cached puzzle description for the given day, downloading if necessary.
#[allow(dead_code)]
pub fn ensure_puzzle(day: u32) -> String {
    let path = format!("inputs/day{:02}.puzzle.txt", day);
    if Path::new(&path).exists() {
        return path;
    }

    let session = read_session();
    fs::create_dir_all("inputs").expect("failed to create inputs directory");

    let url = format!("https://adventofcode.com/2025/day/{}", day);
    let client = reqwest::blocking::Client::new();
    let html = client
        .get(&url)
        .header("Cookie", format!("session={}", session.trim()))
        .header("User-Agent", "github.com/aoc2025rust")
        .send()
        .unwrap_or_else(|e| panic!("failed to fetch puzzle for day {}: {}", day, e))
        .text()
        .unwrap_or_else(|e| panic!("failed to read puzzle page for day {}: {}", day, e));

    let text = extract_articles(&html);
    fs::write(&path, &text)
        .unwrap_or_else(|e| panic!("failed to write puzzle file {}: {}", path, e));

    println!("Downloaded puzzle description for day {}", day);
    path
}

#[allow(dead_code)]
fn extract_articles(html: &str) -> String {
    let mut out = String::new();
    let mut pos = 0;
    while let Some(start) = html[pos..].find("<article") {
        let start = pos + start;
        if let Some(end) = html[start..].find("</article>") {
            let article_html = &html[start..start + end + "</article>".len()];
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(&strip_tags(article_html));
            pos = start + end + "</article>".len();
        } else {
            break;
        }
    }
    out
}

#[allow(dead_code)]
fn strip_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    // collapse runs of blank lines
    let mut result = String::new();
    let mut blank_lines = 0;
    for line in out.lines() {
        if line.trim().is_empty() {
            blank_lines += 1;
            if blank_lines <= 1 {
                result.push('\n');
            }
        } else {
            blank_lines = 0;
            result.push_str(line.trim_end());
            result.push('\n');
        }
    }
    result
}

pub fn read_session() -> String {
    if let Ok(session) = std::env::var("AOC_SESSION") {
        return session;
    }
    fs::read_to_string(".aoc_session")
        .expect("session cookie not found: set AOC_SESSION env var or create a .aoc_session file")
}
