use std::fs;
use itertools::Itertools;
use mysql::{Pool, PooledConn};
use mysql::prelude::Queryable;
use tokio::sync::MutexGuard;
use crate::data_models::{SqlStream, ToCompare};

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
    return unlocked.query_map("SELECT * FROM `items_data`", |(id, name, brand, description, group_type, price, image_path, available_quantity)| {
        SqlStream {
            id,
            name,
            brand,
            description,
            group_type,
            price,
            image_path,
            available_quantity
        }
    })
}

pub fn select_group_type_from_table(unlocked : &mut MutexGuard<PooledConn>) -> mysql::Result<Vec<ToCompare>> {
    return unlocked.query_map("SELECT group_type FROM items_data", |group_type| {
        ToCompare{ compared: group_type }
    },
    )
}

pub fn all_from_table_where_group_type(unlocked : &mut MutexGuard<PooledConn>, where_expression : String) -> mysql::Result<Vec<SqlStream>> {
    return unlocked.query_map(format!(r#"SELECT * FROM `items_data` WHERE group_type = "{}""#, where_expression),
                              |(id, name, brand, description, group_type, price, image_path, available_quantity)| {
                                  SqlStream {
                                      id,
                                      name,
                                      brand,
                                      description,
                                      group_type,
                                      price,
                                      image_path,
                                      available_quantity
                                  }
                              }
    )
}