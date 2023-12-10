#![feature(array_chunks)]
#![feature(array_windows)]
#![feature(box_into_inner)]
#![feature(byte_slice_trim_ascii)]
#![feature(coroutines)]
#![feature(generic_const_exprs)]
#![feature(get_many_mut)]
#![feature(iter_from_coroutine)]
#![feature(let_chains)]
#![feature(never_type)]
#![feature(slice_partition_dedup)]
#![feature(stmt_expr_attributes)]

#![allow(incomplete_features)]

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
);
