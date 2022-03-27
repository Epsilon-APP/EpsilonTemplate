use crate::templates::resource::Resource;
use rocket::serde::json::Value;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub parent: String,
    pub slots: u16,
    pub resources: Resource,
    pub labels: HashMap<String, Value>,
}
