use typeshare::typeshare;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Codable, CaseIterable")]
#[serde(rename_all = "lowercase")]
pub enum AssetRank {
    High = 100,
    Medium = 50,
    Low = 25,
    Trivial = 15,
    Unknown = 0,
    Inactive = -2,
    Abandoned = -5,
    Suspended = -8,
    Migrated = -10,
    Deprecated = -12,
    Spam = -15,
    Fradulent = -20,
}