#![feature(array_windows)]
#![feature(auto_traits)]
#![feature(decl_macro)]
#![feature(hash_raw_entry)]
#![feature(let_chains)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(maybe_uninit_uninit_array)]
#![feature(negative_impls)]
#![feature(trait_alias)]
#![feature(trusted_len)]

pub mod astr;
pub mod cbuffer;
pub mod error;
pub mod graph;
pub mod grid;
pub mod inputs;
pub mod iter;
pub mod ocr;
pub mod offsets;
pub mod outputs;
pub mod parsers;
pub mod prelude;
pub mod result;
pub mod runner;
pub mod util;
pub mod vecs;

pub use paste::paste;
