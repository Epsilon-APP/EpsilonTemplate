use rocket::serde::Deserialize;
use rocket::serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Resources {
    pub minimum: ResourcesInfo,
    pub maximum: ResourcesInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ResourcesInfo {
    pub cpu: f32,
    pub ram: u32,
}
