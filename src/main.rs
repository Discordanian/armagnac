use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;
use terminal_size::{terminal_size, Width};

/// Command-line anagram solver
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// The letters to find anagrams for
    letters: String,

    /// Path to a custom dictionary file (overrides the system dictionary)
    #[arg(short, long)]
    dictionary: Option<PathBuf>,
}

const CANDIDATE_DICT_PATHS: &[&str] = &[
    "/usr/share/dict/words",
    "/usr/local/share/dict/words",
    "/usr/dict/words",
    "/usr/local/dict/words",
];

fn find_dictionary(override_path: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(path) = override_path {
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
        return Err(format!(
            "specified dictionary file not found: {}",
            path.display()
        ));
    }

    for candidate in CANDIDATE_DICT_PATHS {
        let path = Path::new(candidate);
        if path.is_file() {
            return Ok(path.to_path_buf());
        }
    }

    Err(format!(
        "no system dictionary found; tried: {}",
        CANDIDATE_DICT_PATHS.join(", ")
    ))
}

fn sorted_chars(s: &str) -> Vec<char> {
    let mut chars: Vec<char> = s.chars().collect();
    chars.sort_unstable();
    chars
}

fn has_wildcard(letters: &str) -> bool {
    letters.contains('?')
}

/// Returns true if `word` matches `pattern` positionally.
/// `?` in the pattern matches any single character; all other characters must match exactly.
fn matches_pattern(word: &str, pattern: &str) -> bool {
    let mut wc = word.chars();
    let mut pc = pattern.chars();
    loop {
        match (wc.next(), pc.next()) {
            (Some(w), Some('?')) => { let _ = w; }
            (Some(w), Some(p)) => {
                if w != p {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn find_anagrams(letters: &str, dict_path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(dict_path)
        .map_err(|e| format!("failed to read dictionary {}: {e}", dict_path.display()))?;

    let lower = letters.to_lowercase();
    let target_len = letters.chars().count();

    let anagrams = if has_wildcard(letters) {
        content
            .lines()
            .filter(|word| {
                word.chars().count() == target_len
                    && matches_pattern(&word.to_lowercase(), &lower)
            })
            .map(str::to_owned)
            .collect()
    } else {
        let key = sorted_chars(&lower);
        content
            .lines()
            .filter(|word| {
                word.chars().count() == target_len
                    && word.to_lowercase() != lower
                    && sorted_chars(&word.to_lowercase()) == key
            })
            .map(str::to_owned)
            .collect()
    };

    Ok(anagrams)
}

/// Generates all unique permutations of the given characters in lexicographic order.
fn all_permutations(letters: &str) -> Vec<String> {
    let mut chars: Vec<char> = letters.to_lowercase().chars().collect();
    chars.sort_unstable();

    let mut results = Vec::new();
    let mut current = Vec::with_capacity(chars.len());
    let mut used = vec![false; chars.len()];
    permute(&chars, &mut used, &mut current, &mut results);
    results
}

fn permute(
    chars: &[char],
    used: &mut Vec<bool>,
    current: &mut Vec<char>,
    results: &mut Vec<String>,
) {
    if current.len() == chars.len() {
        results.push(current.iter().collect());
        return;
    }
    for i in 0..chars.len() {
        if used[i] {
            continue;
        }
        // Skip duplicate characters at the same depth to avoid repeated permutations
        if i > 0 && chars[i] == chars[i - 1] && !used[i - 1] {
            continue;
        }
        used[i] = true;
        current.push(chars[i]);
        permute(chars, used, current, results);
        current.pop();
        used[i] = false;
    }
}

fn print_columns(words: &[String]) {
    const FALLBACK_WIDTH: usize = 80;
    const COL_GAP: usize = 2;

    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(FALLBACK_WIDTH);

    let col_width = words.iter().map(|w| w.len()).max().unwrap_or(0) + COL_GAP;
    let num_cols = (term_width / col_width).max(1);

    for (i, word) in words.iter().enumerate() {
        if (i + 1) % num_cols == 0 || i + 1 == words.len() {
            println!("{word}");
        } else {
            print!("{word:<col_width$}");
        }
    }
}

fn main() {
    let args = Args::parse();

    let dict_path = find_dictionary(args.dictionary.as_deref()).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    let mut anagrams = find_anagrams(&args.letters, &dict_path).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    if anagrams.is_empty() {
        println!("No anagrams found for \"{}\".", args.letters);
        if !has_wildcard(&args.letters) {
            println!("All letter combinations:");
            let permutations = all_permutations(&args.letters);
            print_columns(&permutations);
        }
    } else {
        anagrams.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        let mut anagrams: Vec<String> = anagrams.iter().map(|w| w.to_lowercase()).collect();
        anagrams.dedup();
        print_columns(&anagrams);
    }
}
