use std::fmt::Debug;
use std::fs;
use itertools::Itertools;
use mysql::{Error, params, Pool, PooledConn};
use mysql::prelude::Queryable;
use tokio::sync::{MutexGuard};
use crate::data_models::{FormedToSendOrder, PlaceOrderBodyJSON, SqlStream, ToCompare, WeightOrPay};

/// /Users/egorivanov/Desktop/mysql.txt
/// r#"C:\Users\User\Desktop\mysql.txt"#

pub fn establish_connection() -> PooledConn {
    let pool = Pool::new(fs::read_to_string(r#"/Users/egorivanov/Desktop/mysql.txt"#).unwrap().trim()).expect("Couldn't connect to a base");
    println!("Connection with MySQL pool is established!");
    return pool.get_conn().unwrap();
}

pub fn remove_repeating_elements_to_string(vector : &Vec<SqlStream>) -> Vec<String> {
    return vector.iter().map(|value| value.group_type.to_string()).collect::<Vec<String>>().into_iter().unique().collect::<Vec<String>>()
}

pub fn select_all_from_table(unlocked : &mut MutexGuard<PooledConn>) -> mysql::Result<Vec<SqlStream>> { // Basic request for mysql base to get all values.
    return unlocked.query_map("SELECT * FROM `items_data`", |(id, name, brand, description, group_type, price, image_path, available_quantity, width_mm, height_mm, weight_piece_grams)| {
        SqlStream {
            id,
            name,
            brand,
            description,
            group_type,
            price,
            image_path,
            available_quantity,
            width_mm,
            height_mm,
            weight_piece_grams
        }
    })
}

pub fn select_group_type_from_table(unlocked : &mut MutexGuard<PooledConn>) -> mysql::Result<Vec<ToCompare>> {
    return unlocked.query_map("SELECT group_type FROM items_data", |group_type| {
        ToCompare{ compared: group_type }
    })
}

pub fn all_from_table_where_group_type(unlocked : &mut MutexGuard<PooledConn>, where_expression : String) -> mysql::Result<Vec<SqlStream>> {
    return unlocked.query_map(format!(r#"SELECT * FROM `items_data` WHERE group_type = "{}""#, where_expression),
                              |(id, name, brand, description, group_type, price, image_path, available_quantity, width_mm, height_mm, weight_piece_grams)| {
                                  SqlStream {
                                      id,
                                      name,
                                      brand,
                                      description,
                                      group_type,
                                      price,
                                      image_path,
                                      available_quantity,
                                      width_mm,
                                      height_mm,
                                      weight_piece_grams
                                  }
                              })
}

pub fn select_from_table_by_id(unlocked : &mut MutexGuard<PooledConn>, id : u16) -> mysql::Result<Vec<SqlStream>> {
    return unlocked.query_map(format!("SELECT * FROM `items_data` WHERE id = {}", id),
                              |(id, name, brand, description, group_type, price, image_path, available_quantity, width_mm, height_mm, weight_piece_grams)| {
                                  SqlStream {
                                      id,
                                      name,
                                      brand,
                                      description,
                                      group_type,
                                      price,
                                      image_path,
                                      available_quantity,
                                      width_mm,
                                      height_mm,
                                      weight_piece_grams
                                  }
                              }
    )
}

pub fn pick_3_random_recommendations(unlocked : &mut MutexGuard<PooledConn>, id : u16, category : String) -> mysql::Result<Vec<SqlStream>> {
    return unlocked.query_map(format!(r#"SELECT * FROM `items_data` WHERE group_type = "{}" AND id != {} ORDER BY RAND() LIMIT 3"#, category, id),
                              |(id, name, brand, description, group_type, price, image_path, available_quantity, width_mm, height_mm, weight_piece_grams)| {
                                  SqlStream {
                                      id,
                                      name,
                                      brand,
                                      description,
                                      group_type,
                                      price,
                                      image_path,
                                      available_quantity,
                                      width_mm,
                                      height_mm,
                                      weight_piece_grams
                                  }
                              }
    )
}

fn get_full_price_or_weight<T : PartialEq + std::iter::Sum<f32>>(orient : &PlaceOrderBodyJSON, switcher : WeightOrPay) -> T {
    match switcher {
        WeightOrPay::Weight => {orient.contents.iter().map(|value| value.total_weight_grams).sum()}
        WeightOrPay::TotalPay => {orient.contents.iter().map(|value| value.total_price).sum()}
    }
}

pub fn insert_an_order(order_body : PlaceOrderBodyJSON, unlocked : &mut MutexGuard<PooledConn>) -> Result<(), Error> {
    let total_to_pay = get_full_price_or_weight(&order_body, WeightOrPay::TotalPay);
    let total_weight = get_full_price_or_weight(&order_body, WeightOrPay::Weight);

    let mut to_send : Vec<FormedToSendOrder> = Vec::with_capacity(1);
    to_send.push(FormedToSendOrder {
        order_id: 0,
        order_status: "ACTIVE".to_string(),
        contents: order_body.contents,
        total_to_pay: total_to_pay,
        total_weight: total_weight,
        phone: order_body.phone,
        email: order_body.email,
        delivery_type: order_body.delivery_type,
        delivery_address: order_body.delivery_address,
    });

    return unlocked.exec_batch(r"INSERT INTO orders_data VALUES (:order_id, :order_status, :contents, :total_to_pay, :total_weight, :phone, :email, :delivery_type, :delivery_address, NOW(), NOW())",
    to_send.iter().map(|obj| params! {
        "order_id" => obj.order_id,
        "order_status" => &obj.order_status,
        "contents" => serde_json::to_string(&obj.contents).unwrap(),
        "total_to_pay" => obj.total_to_pay,
        "total_weight" => obj.total_weight,
        "phone" => &obj.phone,
        "email" => &obj.email,
        "delivery_type" => &obj.delivery_type,
        "delivery_address" => &obj.delivery_address
    }))
}