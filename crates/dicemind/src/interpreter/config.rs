use serde::{Deserialize, Serialize};

use crate::syntax::PositiveInteger;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RollerConfig {
    assumed_quantity: PositiveInteger,
    assumed_power: PositiveInteger,
}

impl Default for RollerConfig {
    fn default() -> Self {
        Self {
            assumed_quantity: 1u32.into(),
            assumed_power: 6u32.into(),
        }
    }
}

impl RollerConfig {
    pub fn quantity(&self) -> PositiveInteger {
        self.assumed_quantity.clone()
    }

    pub fn power(&self) -> PositiveInteger {
        self.assumed_power.clone()
    }
}
