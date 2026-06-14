use std::path::{Path, PathBuf};
use std::process;

use clap::Parser;

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

fn main() {
    let args = Args::parse();

    let dict_path = find_dictionary(args.dictionary.as_deref()).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    println!("Using dictionary: {}", dict_path.display());
    println!("Finding anagrams for: {}", args.letters);
}
