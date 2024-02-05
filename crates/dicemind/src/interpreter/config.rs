use serde::{Deserialize, Serialize};

use crate::syntax::PositiveInteger;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RollerConfig {
    default_count: PositiveInteger,
    default_power: PositiveInteger,
}

impl Default for RollerConfig {
    fn default() -> Self {
        Self {
            default_count: 1u32.into(),
            default_power: 6u32.into(),
        }
    }
}

impl RollerConfig {
    pub fn count(&self) -> PositiveInteger {
        self.default_count.clone()
    }

    pub fn power(&self) -> PositiveInteger {
        self.default_power.clone()
    }
}
