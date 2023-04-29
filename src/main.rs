use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use strsim::levenshtein;
use regex::Regex;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut latex_citations: Vec<String> = Vec::new();
    let mut bibtex_keys: Vec<String> = Vec::new();

    for filename in &args {
        if filename.ends_with(".tex") {
            let citations = extract_citations_latex(Path::new(&filename))?;
            latex_citations.extend(citations);
        } else if filename.ends_with(".bib") {
            let keys = extract_bibtex_keys(Path::new(&filename))?;
            bibtex_keys.extend(keys);
        } else {
            println!("Unknown file type: {}", filename);
        }
    }

    let diff: Vec<_> = latex_citations.iter().filter(|key| !bibtex_keys.contains(key)).collect();
    for unmatched_key in diff {
        let most_similar = find_nearest(&unmatched_key, &bibtex_keys);
        println!("{} not found in bib. Most similar key in bib: {}", unmatched_key, most_similar);
    }

    Ok(())
}

fn extract_citations_latex(filepath: &Path) -> io::Result<Vec<String>> {
    let file = File::open(&filepath)?;
    let reader = io::BufReader::new(file);
    let cite_re = Regex::new(r#"\\(?:cite[^{]*?\{([^}]+)\})"#).unwrap();
    let split_re = Regex::new(r#"\s*,\s*"#).unwrap();
    
    let mut citations = Vec::new();

    for line in reader.lines() {
        let line = line?;
        for m in cite_re.captures_iter(&line) {
            let captured = m.get(1).unwrap().as_str();
            let split_captures: Vec<String> = split_re.split(captured).map(|s| s.trim().to_owned()).collect();
            citations.extend(split_captures)
        }
    }

    Ok(citations)
}

fn extract_bibtex_keys(filepath: &Path) -> io::Result<Vec<String>> {
    let file = File::open(&filepath)?;
    let reader = io::BufReader::new(file);
    let bibtex_re = Regex::new(r#"^\s*\@[\w]+\{([^\,]+),"#).unwrap();

    let mut keys = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if let Some(captures) = bibtex_re.captures(&line) {
            if let Some(m) = captures.get(1) {
                keys.push(m.as_str().trim().to_owned());
            }
        }
    }

    Ok(keys)
}

fn find_nearest(key: &str, keys: &[String]) -> String {
    let empty_str = "".to_string();
    let (nearest, _) = keys.iter()
        .map(|k| (k, levenshtein(key, k)))
        .min_by_key(|(_, dist)| *dist)
        .unwrap_or_else(|| (&empty_str, 0));

    nearest.clone()
}
