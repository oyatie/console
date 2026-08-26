use serde::{Deserialize, Serialize};

use crate::caps::SurfaceCaps;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub user_id: String,
    pub display_name: String,
    pub caps: SurfaceCaps,
}

impl Session {
    #[must_use]
    pub fn unauthenticated() -> Option<Self> {
        None
    }
}
