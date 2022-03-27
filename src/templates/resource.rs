use rocket::serde::Deserialize;
use rocket::serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Resource {
    pub minimum: ResourceInfo,
    pub maximum: ResourceInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceInfo {
    pub cpu: u8,
    pub ram: u32,
}
