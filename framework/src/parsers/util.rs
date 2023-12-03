use crate::astr::AStr;
use std::slice::Split;

pub trait AStrExt {
    fn lines(&self) -> Split<'_, u8, for<'r> fn(&'r u8) -> bool>;
    fn split_space(&self) -> Split<'_, u8, for<'r> fn(&'r u8) -> bool>;
}

impl AStrExt for AStr {
    fn lines(&self) -> Split<'_, u8, for<'r> fn(&'r u8) -> bool> {
        self.split(|&l| l == b'\n')
    }
    fn split_space(&self) -> Split<'_, u8, for<'r> fn(&'r u8) -> bool> {
        self.split(|&l| l == b' ')
    }
}
