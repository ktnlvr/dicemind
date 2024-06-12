use serde::{Deserialize, Serialize};

use crate::syntax::PositiveInteger;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RollerConfig {
    assumed_quantity: PositiveInteger,
    assumed_power: PositiveInteger,
    chain_explosions: bool,
}

impl Default for RollerConfig {
    fn default() -> Self {
        Self {
            assumed_quantity: 1u32.into(),
            assumed_power: 6u32.into(),
            chain_explosions: false,
        }
    }
}

impl RollerConfig {
    pub fn chain_explosions(&self) -> bool {
        return self.chain_explosions
    }

    pub fn quantity(&self) -> PositiveInteger {
        self.assumed_quantity.clone()
    }

    pub fn power(&self) -> PositiveInteger {
        self.assumed_power.clone()
    }
}
