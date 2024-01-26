#![deny(unsafe_op_in_unsafe_fn)]
#![feature(box_patterns)]
#![feature(result_flattening)]

pub mod interpreter;
pub mod parser;
mod visitor;

pub mod prelude {
    pub use crate::interpreter::FastRoller;
    pub use crate::parser::parse;
    pub use crate::visitor::Visitor;
}
