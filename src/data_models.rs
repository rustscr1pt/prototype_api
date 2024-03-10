use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct CatalogMainRequest { // An answer for all positions request
    pub total_items : u16,
    pub list_of_groups : Vec<String>,
    pub all_items : Vec<SqlStream>
}
#[derive(Debug, Serialize)]
pub struct IndexBasicRequest { // An answer for a basic request from index.html
    pub random_positions : Vec<SqlStream>,
    pub available_categories : Vec<CategoryMainRequest>,
}
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SqlStream { // A data for one object
    pub id : u16,
    pub name : String,
    pub brand : String,
    pub description : String,
    pub group_type : String,
    pub price : u32,
    pub image_path : String,
    pub available_quantity : u32,
    pub width_mm : u32,
    pub height_mm : u32,
    pub weight_piece_grams : u32
}
#[derive(Debug, Serialize)]
pub struct ToCompare {
    pub compared : String
}
#[derive(Debug, Serialize)]
pub struct Message {
    pub reply : String
}
#[derive(Debug, Serialize, Clone)]
pub struct CategoryMainRequest {
    pub category : String,
    pub amount : u16
}
#[derive(Debug, Serialize, Clone)]
pub struct ConcreteItemLayout { // An answer for concrete item to display at concrete.html
    pub item : SqlStream,
    pub recommendations : Vec<SqlStream>
}
#[derive(Debug, Deserialize)]
pub struct PlaceOrderBodyJSON {
    pub contents : Vec<ItemSampleToDecode>,
    pub phone : String,
    pub email : String,
    pub delivery_type : String,
    pub delivery_address : String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemSampleToDecode {
    pub position : InnerItemDataDeserialize
}

pub enum WeightOrPay {
    Weight,
    TotalPay
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InnerItemDataDeserialize {
    pub item : SqlStream,
    pub quantity : u16,
    pub total_price : f32,
    pub total_weight_grams : f32
}

#[derive(Debug)]
pub struct FormedToSendOrder {
    pub order_id : u16,
    pub order_status : String,
    pub contents : Vec<ItemSampleToDecode>,
    pub total_to_pay : f32,
    pub total_weight : f32,
    pub phone : String,
    pub email : String,
    pub delivery_type : String,
    pub delivery_address : String,
}