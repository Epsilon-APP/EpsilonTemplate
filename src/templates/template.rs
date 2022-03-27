use crate::parents::parent::Type;
use crate::templates::resources::Resources;
use rocket::serde::json::Value;
use rocket::serde::Deserialize;
use rocket::serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub parent: String,

    #[serde(rename = "type")]
    pub t: Option<Type>,
    pub slots: u16,
    pub resources: Resources,
    pub labels: HashMap<String, Value>,
}
