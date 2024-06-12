#![deny(unsafe_op_in_unsafe_fn)]
#![feature(result_flattening)]
#![feature(concat_idents)]
#![feature(box_patterns)]
#![feature(extract_if)]
#![feature(is_sorted)]

pub mod interpreter;
pub mod parser;
pub mod syntax;
mod options;
mod simplify;
mod visitor;

pub mod prelude {
    pub use crate::interpreter::StandardNaiveRoller;
    pub use crate::parser::{parse, ParsingError};
    pub use crate::options::RollerOptions;
    pub use crate::syntax::Expression;
    pub use crate::simplify::advanced_simplify;
}
