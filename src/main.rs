use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;
use terminal_size::{terminal_size, Width};

/// Command-line anagram solver
#[derive(Parser, Debug)]
#[command(version, about)]
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
    letters.contains('?') || letters.contains('*') || letters.contains('-')
}

/// Returns true if every character in `word` can be matched to a distinct character in `base`.
fn is_submultiset(word: &str, base: &[char]) -> bool {
    let mut remaining = base.to_vec();
    for ch in word.chars() {
        match remaining.iter().position(|&c| c == ch) {
            Some(pos) => { remaining.remove(pos); }
            None => return false,
        }
    }
    true
}

/// Positional match: each `?` marks the slot where a wild (new) character must appear.
/// The fixed characters may occupy the non-`?` slots in any order.
fn matches_positional(word: &str, pattern: &str) -> bool {
    let word_chars: Vec<char> = word.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    if word_chars.len() != pattern_chars.len() {
        return false;
    }

    // The chars at non-`?` positions in the word must be an anagram of the fixed chars.
    let mut fixed: Vec<char> = pattern_chars.iter().filter(|&&c| c != '?').copied().collect();
    let mut word_fixed: Vec<char> = word_chars
        .iter()
        .zip(pattern_chars.iter())
        .filter(|&(_, p)| *p != '?')
        .map(|(&w, _)| w)
        .collect();

    fixed.sort_unstable();
    word_fixed.sort_unstable();

    word_fixed == fixed
}

/// Non-positional match: the word must contain every required char (multiset).
/// `?` and `*` in the pattern both act as free-char wildcards that consume one slot each.
fn matches_nonpositional(word: &str, required: &[char]) -> bool {
    let mut remaining: Vec<char> = word.chars().collect();
    for &ch in required {
        match remaining.iter().position(|&c| c == ch) {
            Some(pos) => { remaining.remove(pos); }
            None => return false,
        }
    }
    true
}

fn find_anagrams(letters: &str, dict_path: &Path) -> Result<Vec<String>, String> {
    let content = fs::read_to_string(dict_path)
        .map_err(|e| format!("failed to read dictionary {}: {e}", dict_path.display()))?;

    let lower = letters.to_lowercase();
    let target_len = letters.chars().count();

    let n_dash = lower.chars().filter(|&c| c == '-').count();

    let anagrams = if n_dash > 0 {
        // Remove n_dash letters from the pool and find all anagrams of any resulting subset.
        let base: Vec<char> = lower.chars().filter(|&c| c != '-').collect();
        let target_len = base.len().saturating_sub(n_dash);
        content
            .lines()
            .filter(|word| {
                word.chars().count() == target_len
                    && is_submultiset(&word.to_lowercase(), &base)
            })
            .map(str::to_owned)
            .collect()
    } else if lower.contains('*') {
        // Non-positional: letters must appear somewhere in the word; * and ? are free slots.
        let required: Vec<char> = lower.chars().filter(|&c| c != '?' && c != '*').collect();
        content
            .lines()
            .filter(|word| {
                word.chars().count() == target_len
                    && matches_nonpositional(&word.to_lowercase(), &required)
            })
            .map(str::to_owned)
            .collect()
    } else if lower.contains('?') {
        // Positional: each character must match its exact position; ? accepts any char.
        content
            .lines()
            .filter(|word| {
                word.chars().count() == target_len
                    && matches_positional(&word.to_lowercase(), &lower)
            })
            .map(str::to_owned)
            .collect()
    } else {
        // Pure anagram: sorted character keys must match.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Write a small word list to a temp file and return it.
    /// The file stays alive as long as the returned handle is in scope.
    fn make_dict(words: &[&str]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "{}", words.join("\n")).unwrap();
        f
    }

    fn anagrams_of(letters: &str, words: &[&str]) -> Vec<String> {
        let dict = make_dict(words);
        let mut results = find_anagrams(letters, dict.path()).unwrap();
        results.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        let mut results: Vec<String> = results.iter().map(|w| w.to_lowercase()).collect();
        results.dedup_by(|a, b| a == b);
        results
    }

    // ── sorted_chars ─────────────────────────────────────────────────────────

    #[test]
    fn sorted_chars_basic() {
        assert_eq!(sorted_chars("cba"), vec!['a', 'b', 'c']);
    }

    #[test]
    fn sorted_chars_with_repeats() {
        assert_eq!(sorted_chars("aab"), vec!['a', 'a', 'b']);
    }

    // ── has_wildcard ─────────────────────────────────────────────────────────

    #[test]
    fn has_wildcard_plain() {
        assert!(!has_wildcard("silent"));
    }

    #[test]
    fn has_wildcard_question_mark() {
        assert!(has_wildcard("si?ent"));
    }

    #[test]
    fn has_wildcard_star() {
        assert!(has_wildcard("si*ent"));
    }

    #[test]
    fn has_wildcard_dash() {
        assert!(has_wildcard("silent-"));
    }

    // ── is_submultiset ───────────────────────────────────────────────────────

    #[test]
    fn submultiset_exact_match() {
        let base: Vec<char> = "silent".chars().collect();
        assert!(is_submultiset("inlet", &base));
    }

    #[test]
    fn submultiset_char_not_in_base() {
        let base: Vec<char> = "silent".chars().collect();
        assert!(!is_submultiset("proxy", &base));
    }

    #[test]
    fn submultiset_repeated_char_within_limit() {
        let base: Vec<char> = "aab".chars().collect();
        assert!(is_submultiset("aa", &base));
    }

    #[test]
    fn submultiset_repeated_char_exceeds_limit() {
        let base: Vec<char> = "aab".chars().collect();
        assert!(!is_submultiset("aaa", &base));
    }

    // ── matches_positional ───────────────────────────────────────────────────

    #[test]
    fn positional_wild_char_at_correct_slot() {
        // fr?st: fixed chars {f,r,s,t} can be anywhere except position 2
        assert!(matches_positional("frost", "fr?st"));
        assert!(matches_positional("frist", "fr?st"));
    }

    #[test]
    fn positional_rejects_fixed_char_at_wild_slot() {
        // "first": position 2 is 'r', which is in {f,r,s,t} — should not match
        assert!(!matches_positional("first", "fr?st"));
        assert!(!matches_positional("forst", "fr?st"));
    }

    #[test]
    fn positional_fixed_chars_can_reorder() {
        // st?fr has the same fixed set as fr?st
        assert!(matches_positional("frost", "st?fr"));
        assert!(matches_positional("frist", "st?fr"));
    }

    #[test]
    fn positional_length_mismatch() {
        assert!(!matches_positional("frost", "fr?s"));
    }

    // ── matches_nonpositional ────────────────────────────────────────────────

    #[test]
    fn nonpositional_required_chars_present() {
        let required: Vec<char> = "tb".chars().collect();
        assert!(matches_nonpositional("bat", &required));
        assert!(matches_nonpositional("tab", &required));
        assert!(matches_nonpositional("but", &required));
    }

    #[test]
    fn nonpositional_missing_required_char() {
        let required: Vec<char> = "tb".chars().collect();
        assert!(!matches_nonpositional("cat", &required));
    }

    #[test]
    fn nonpositional_repeated_required_char() {
        let required: Vec<char> = "tt".chars().collect();
        assert!(matches_nonpositional("att", &required));
        assert!(!matches_nonpositional("cat", &required)); // only one t
    }

    // ── all_permutations ─────────────────────────────────────────────────────

    #[test]
    fn permutations_unique_chars() {
        let mut p = all_permutations("abc");
        p.sort();
        assert_eq!(p, vec!["abc", "acb", "bac", "bca", "cab", "cba"]);
    }

    #[test]
    fn permutations_repeated_chars_no_duplicates() {
        let p = all_permutations("aab");
        // unique permutations: aab, aba, baa
        assert_eq!(p, vec!["aab", "aba", "baa"]);
    }

    #[test]
    fn permutations_single_char() {
        assert_eq!(all_permutations("x"), vec!["x"]);
    }

    // ── find_anagrams (integration) ──────────────────────────────────────────

    #[test]
    fn find_pure_anagram() {
        let words = &["listen", "silent", "enlist", "tinsel", "inlets", "hello"];
        let results = anagrams_of("silent", words);
        assert_eq!(results, vec!["enlist", "inlets", "listen", "tinsel"]);
    }

    #[test]
    fn find_anagram_excludes_input_word() {
        let words = &["silent", "listen"];
        let results = anagrams_of("silent", words);
        assert!(!results.contains(&"silent".to_string()));
        assert!(results.contains(&"listen".to_string()));
    }

    #[test]
    fn find_anagram_case_insensitive() {
        let words = &["Listen", "ENLIST", "hello"];
        let results = anagrams_of("silent", words);
        assert!(results.contains(&"listen".to_string()));
        assert!(results.contains(&"enlist".to_string()));
    }

    #[test]
    fn find_anagram_none_found() {
        let words = &["hello", "world"];
        let results = anagrams_of("xyz", words);
        assert!(results.is_empty());
    }

    #[test]
    fn find_positional_wildcard() {
        let words = &["frost", "frist", "first", "forst", "hello"];
        let results = anagrams_of("fr?st", words);
        assert_eq!(results, vec!["frist", "frost"]);
    }

    #[test]
    fn find_positional_wildcard_reordered_pattern() {
        let words = &["frost", "frist", "first", "forst"];
        let results_frqst = anagrams_of("fr?st", words);
        let results_stqfr = anagrams_of("st?fr", words);
        assert_eq!(results_frqst, results_stqfr);
    }

    #[test]
    fn find_nonpositional_wildcard() {
        let words = &["bat", "bet", "bit", "tab", "tub", "cat", "hello"];
        let results = anagrams_of("t*b", words);
        assert!(results.contains(&"bat".to_string()));
        assert!(results.contains(&"tab".to_string()));
        assert!(results.contains(&"tub".to_string()));
        assert!(!results.contains(&"cat".to_string()));
    }

    #[test]
    fn find_with_dash_removes_one_letter() {
        // silent- should find 5-letter words that are subsets of {s,i,l,e,n,t}
        let words = &["inlet", "inset", "stile", "hello", "silent"];
        let results = anagrams_of("silent-", words);
        assert!(results.contains(&"inlet".to_string()));
        assert!(results.contains(&"inset".to_string()));
        assert!(results.contains(&"stile".to_string()));
        assert!(!results.contains(&"hello".to_string()));
        assert!(!results.contains(&"silent".to_string())); // wrong length
    }

    #[test]
    fn find_with_two_dashes_removes_two_letters() {
        let words = &["tile", "line", "sine", "silent", "stile", "hello"];
        let results = anagrams_of("silent--", words);
        assert!(results.contains(&"tile".to_string()));
        assert!(results.contains(&"line".to_string()));
        assert!(results.contains(&"sine".to_string()));
        assert!(!results.contains(&"stile".to_string())); // 5 letters, too long
        assert!(!results.contains(&"hello".to_string())); // chars not in pool
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
