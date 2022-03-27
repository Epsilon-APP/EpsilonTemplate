use rocket::serde::Deserialize;
use rocket::serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Parent {
    pub name: String,

    #[serde(rename = "type")]
    pub t: Type,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub enum Type {
    Server,
    Proxy,
}
