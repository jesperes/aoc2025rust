use std::collections::HashMap;
use std::fs;

fn parse(filename: &str) -> HashMap<String, Vec<String>> {
    let mut graph = HashMap::new();
    for line in fs::read_to_string(filename).unwrap().lines() {
        let (name, rest) = line.split_once(": ").unwrap();
        let neighbors: Vec<String> = rest.split_whitespace().map(|s| s.to_string()).collect();
        graph.insert(name.to_string(), neighbors);
    }
    graph
}

fn count_paths(graph: &HashMap<String, Vec<String>>, node: &str, memo: &mut HashMap<String, u64>) -> u64 {
    if node == "out" { return 1; }
    if let Some(&v) = memo.get(node) { return v; }
    let count = graph.get(node).map_or(0, |neighbors| {
        neighbors.iter().map(|n| count_paths(graph, n, memo)).sum()
    });
    memo.insert(node.to_string(), count);
    count
}

pub fn solve_part1(filename: &str) -> u64 {
    let graph = parse(filename);
    let mut memo = HashMap::new();
    count_paths(&graph, "you", &mut memo)
}

fn count_paths2(graph: &HashMap<String, Vec<String>>, node: &str, mask: u8, memo: &mut HashMap<(String, u8), u64>) -> u64 {
    let new_mask = mask
        | if node == "dac" { 1 } else { 0 }
        | if node == "fft" { 2 } else { 0 };

    if node == "out" {
        return if new_mask == 3 { 1 } else { 0 };
    }

    let key = (node.to_string(), new_mask);
    if let Some(&v) = memo.get(&key) { return v; }

    let count = graph.get(node).map_or(0, |neighbors| {
        neighbors.iter().map(|n| count_paths2(graph, n, new_mask, memo)).sum()
    });
    memo.insert(key, count);
    count
}

pub fn solve_part2(filename: &str) -> u64 {
    let graph = parse(filename);
    let mut memo = HashMap::new();
    count_paths2(&graph, "svr", 0, &mut memo)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE1: &str = "\
aaa: you hhh
you: bbb ccc
bbb: ddd eee
ccc: ddd eee fff
ddd: ggg
eee: out
fff: out
ggg: out
hhh: ccc fff iii
iii: out
";

    const EXAMPLE2: &str = "\
svr: aaa bbb
aaa: fft
fft: ccc
bbb: tty
tty: ccc
ccc: ddd eee
ddd: hub
hub: fff
eee: dac
dac: fff
fff: ggg hhh
ggg: out
hhh: out
";

    #[test]
    fn example() {
        let tmp1 = crate::testutil::TempFile::write(EXAMPLE1);
        let tmp2 = crate::testutil::TempFile::write(EXAMPLE2);
        assert_eq!(solve_part1(tmp1.path()), 5);
        assert_eq!(solve_part2(tmp2.path()), 2);
    }
}
