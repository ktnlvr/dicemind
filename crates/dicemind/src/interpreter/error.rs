use thiserror::Error;

use crate::{
    prelude::Expression,
    syntax::{AnnotationString, Integer},
};

#[derive(Debug, Error)]
pub enum RollerError {
    // The input is too large
    #[error("Input value {value} too large and can't fit inside 2^63 - 1")]
    ValueTooLarge { value: Integer },
    // Error while computing, the computation could not be finished
    #[error("The value has overflown, the result was too large")]
    Overflow,
    #[error(
        "Could not truncate dice rolls, you rolled {rolled} dice but the augments tried to remove {removed}"
    )]
    TruncationFailure { rolled: u32, removed: u32 },
    #[error("The dice roll will always explode")]
    InfiniteExplosion,
    #[error("Annotation \"{annotation}\" denotes two different rolls: {first:?} and {second:?}")]
    DuplicateAnnotation {
        annotation: AnnotationString,
        first: Expression,
        second: Expression,
    },
}

pub type RollerResult<T> = Result<T, RollerError>;
