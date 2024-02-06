use thiserror::Error;

use crate::{prelude::Expression, syntax::AnnotationString};

#[derive(Debug, Error)]
pub enum RollerError {
    #[error("Value too large and can't fit inside 2^31 - 1")]
    ValueTooLarge,
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
