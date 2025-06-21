/// Get Release Status Query
/// 
/// This query aggregates release status information from git, tasks, and AI domains.

#[cfg(feature = "new-domains")]
use std::collections::HashMap;
#[cfg(feature = "new-domains")]
use std::any::Any;
#[cfg(feature = "new-domains")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "new-domains")]
use chrono::{DateTime, Utc};

#[cfg(feature = "new-domains")]
use super::Query;

/// Query to get current release status
#[cfg(feature = "new-domains")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetReleaseStatusQuery {
    pub repository_path: String,
}

#[cfg(feature = "new-domains")]
impl Query for GetReleaseStatusQuery {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
