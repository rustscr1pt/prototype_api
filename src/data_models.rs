use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct CatalogMainRequest { // An answer for all positions request
    pub total_items : u16,
    pub list_of_groups : Vec<String>,
    pub all_items : Vec<SqlStream>
}
#[derive(Debug, Serialize)]
pub struct IndexBasicRequest { // An answer for a basic request from index.html
    pub random_positions : Vec<SqlStream>,
    pub available_categories : Vec<String>,
}
#[derive(Debug, Serialize, Clone)]
pub struct SqlStream { // A data for one object
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
#[derive(Debug, Serialize)]
pub struct CategoryMainRequest {
    pub category : String,
    pub amount : u16
}