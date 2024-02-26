use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SqlStream {
    pub id : u16,
    pub name : String,
    pub image_path : String,
    pub price : u32
}
#[derive(Debug, Serialize)]
pub struct Message {
    pub reply : String
}