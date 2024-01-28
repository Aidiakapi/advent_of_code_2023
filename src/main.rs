#![feature(array_chunks)]
#![feature(byte_slice_trim_ascii)]
#![feature(coroutines)]
#![feature(hash_raw_entry)]
#![feature(iter_from_coroutine)]
#![feature(iter_repeat_n)]
#![feature(map_try_insert)]
#![feature(slice_partition_dedup)]

#![feature(custom_test_frameworks)]
#![cfg_attr(feature = "criterion", test_runner(criterion::runner))]

#![allow(clippy::zero_prefixed_literal)]

mod prelude;

framework::main!(
    day01,
    day02,
    day03,
    day04,
    day05,
    day06,
    day07,
    day08,
    day09,
    day10,
    day11,
    day12,
    day13,
    day14,
    day15,
    day16,
    day17,
    day18,
    day19,
    day20,
    day21,
    day22,
    day23,
    day24,
    day25,
);
