# armagnac

A command-line anagram solver. Give it a word and it returns all dictionary words that use exactly the same letters. Special characters extend the search with wildcards and letter removal.

## Requirements

- Rust toolchain (stable)
- A system dictionary file (standard on macOS and most Linux distributions)

## Installation

```sh
cargo install --path .
```

Or build and run directly from the repo:

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

Results are printed in lowercase, sorted alphabetically, and laid out in equally-spaced columns that fit the terminal width.

## Special characters

The `<LETTERS>` argument supports four modes depending on which special characters are present. The modes are mutually exclusive and checked in priority order.

### No special characters — pure anagram

All letters must be used, no more and no less. The result is every dictionary word whose sorted characters exactly match the sorted input.

```sh
$ armagnac silent
enlist  listen  tinsel
```

If no dictionary words are found, all unique letter permutations are shown as a fallback:

```sh
$ armagnac xyz
No anagrams found for "xyz".
All letter combinations:
xyz  xzy  yxz  yzx  zxy  zyx
```

### `?` — positional wildcard

Each `?` matches any single letter at that exact position. All other characters must appear at their exact positions.

```sh
$ armagnac fr?st
frist  frost
```

```sh
$ armagnac t?b
tab  tib  tub
```

### `*` — non-positional wildcard

Each `*` matches any single letter anywhere in the word. The non-`*`, non-`?` characters must all appear in the word but are not tied to a specific position. When `*` is present, `?` also acts as a non-positional free slot.

```sh
$ armagnac t*b
bat  bet  bit  bot  but  tab  tib  tub
```

```sh
$ armagnac fr*st
first  forst  frist  frost
```

### `-` — letter removal

Each `-` removes one letter from the pool before solving. The result is every dictionary word that can be formed from any subset of the remaining letters at the reduced length.

```sh
$ armagnac silent-
inlet  inset  islet  lenis  neist  snite  stein  stile  ...
```

```sh
$ armagnac silent--
isle  lens  lent  lest  lien  line  lint  list  lite  sine  site  silt  tile  tine  ...
```

## Dictionary

armagnac looks for a system dictionary in the following locations, in order:

1. `/usr/share/dict/words`
2. `/usr/local/share/dict/words`
3. `/usr/dict/words`
4. `/usr/local/dict/words`

Use `-d` / `--dictionary` to supply your own word list. The file must contain one word per line.

If no dictionary can be found, the program exits with an error message.
