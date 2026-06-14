# armagnac

A command-line anagram solver. Give it a word and it returns all dictionary words that use exactly the same letters. When no dictionary words exist, it falls back to listing every unique letter combination.

## Requirements

- Rust toolchain (stable)
- A system dictionary file (standard on macOS and most Linux distributions)

## Installation

```sh
cargo install --path .
```

Or just build and run from the repo:

```sh
cargo build --release
./target/release/armagnac <letters>
```

## Usage

```
armagnac [OPTIONS] <LETTERS>

Arguments:
  <LETTERS>  The letters to find anagrams for

Options:
  -d, --dictionary <DICTIONARY>  Path to a custom dictionary file (overrides the system dictionary)
  -h, --help                     Print help
  -V, --version                  Print version
```

### Examples

Find anagrams of a word:

```sh
$ armagnac silent
enlist  listen  tinsel
```

When no dictionary words are found, all unique letter permutations are shown instead:

```sh
$ armagnac xyz
No anagrams found for "xyz".
All letter combinations:
xyz  xzy  yxz  yzx  zxy  zyx
```

Use a custom dictionary:

```sh
$ armagnac --dictionary /path/to/wordlist.txt silent
```

## Dictionary

armagnac looks for a system dictionary in the following locations, in order:

1. `/usr/share/dict/words` (macOS, most Linux distros)
2. `/usr/local/share/dict/words`
3. `/usr/dict/words`
4. `/usr/local/dict/words`

You can override this with the `-d` / `--dictionary` flag. The file is expected to contain one word per line.

If no dictionary can be found, the program exits with an error message.
