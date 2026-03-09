use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Champion {
    pub id: String,
    pub name: String,
    pub title: String,
    pub tags: Vec<String>,
    pub image_full: String,
}
