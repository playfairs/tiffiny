
pub fn strip_quotes(path: &str) -> String {
    path.trim_matches(|c| c == '"' || c == '\'').to_string()
}
pub fn normalize(path: &str) -> String {
    path.replace('~', &std::env::home_dir().unwrap().display().to_string()).to_string()
}

pub fn calculate_entropy(data: &[u8]) -> f64 {
    let mut counts = [0usize; 256];

    for &b in data {
        counts[b as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count == 0 {
            continue;
        }

        let p = count as f64 / len;
        entropy -= p * p.log2();
    }

    entropy
}