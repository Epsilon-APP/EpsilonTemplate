use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Resource {
    pub minimum: ResourceInfo,
    pub maximum: ResourceInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ResourceInfo {
    pub cpu: u8,
    pub ram: u32
}
