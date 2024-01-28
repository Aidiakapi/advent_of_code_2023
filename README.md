# Advent of Code 2023

[![Rust](https://github.com/Aidiakapi/advent_of_code_2023/actions/workflows/rust.yml/badge.svg)](https://github.com/Aidiakapi/advent_of_code_2023/actions/workflows/rust.yml)

My solutions for Advent of Code 2023. Written in Rust 🦀.

- Clone the repository.
- Make sure you have a nightly version of Rust around January 2024.
- `cargo run --release` for all days, `cargo run --release -- NN` for a specific
  day.
- Want your own inputs?
    - **Auto-download:** Delete the `inputs` directory, then create a
      `session_key.txt` file containing your AoC website's session cookie value.
    - **Manually:** Replace the contents of a `inputs/NN.txt` file with your
      desired input.
- Benchmarks? 🚤
    - `cargo bench --features "criterion"`
    - optionally add `-- dayNN` at the end, to run a specific day!
