#![feature(box_patterns)]
#![feature(result_flattening)]

use crate::{
    interpreter::{convolve, BigRoller},
    parser::{parse, PositiveInteger},
    visitor::Visitor,
};

pub mod interpreter;
pub mod parser;
mod visitor;

pub mod prelude {
    pub use crate::interpreter::FastRoller;
    pub use crate::parser::parse;
    pub use crate::visitor::Visitor;
}