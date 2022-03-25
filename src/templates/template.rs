use std::collections::HashMap;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use crate::templates::resource::Resource;

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub parent: String,
    pub slots: u16,
    pub resources: Resource,
    pub labels: HashMap<String, Value>
}