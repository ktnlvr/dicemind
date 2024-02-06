#![deny(unsafe_op_in_unsafe_fn)]
#![feature(concat_idents)]
#![feature(box_patterns)]
#![feature(extract_if)]
#![feature(result_flattening)]
#![feature(is_sorted)]

pub mod interpreter;
mod minmax;
pub mod parser;
pub mod syntax;
mod visitor;

pub mod prelude {
    pub use crate::interpreter::StandardFastRoller;
    pub use crate::parser::parse;
    pub use crate::syntax::Expression;
    pub use crate::visitor::Visitor;
}
