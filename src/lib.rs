#![doc = include_str!("../README.md")]

pub mod lisp_comb;
pub mod parser_comb;
pub use parser_comb::{parse, Parser};

#[derive(Debug, Clone, PartialEq)]
pub enum LispObject {
    List(Vec<LispObject>),
    String(String),
    Ident(String),
}
