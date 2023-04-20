use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SwipeUserResponse {
    #[serde(rename = "matched")]
    Matched,
    #[serde(rename = "not_matched")]
    NotMatched,
}
