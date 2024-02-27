use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SqlStream {
    pub id : u16,
    pub name : String,
    pub brand : String,
    pub description : String,
    pub group_type : String,
    pub price : u32,
    pub image_path : String,
    pub available_quantity : u32
}
#[derive(Debug, Serialize)]
pub struct ToCompare {
    pub compared : String
}
#[derive(Debug, Serialize)]
pub struct Message {
    pub reply : String
}