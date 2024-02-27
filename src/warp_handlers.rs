use std::sync::Arc;
use mysql::{Error, PooledConn};
use mysql::prelude::Queryable;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};
use warp::reply::json;
use crate::data_models::{Message, SqlStream, ToCompare};

type WebResult<T> = Result<T, Rejection>;

pub async fn get_all_items_catalog(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    let mut unlocked = pool.lock().await;
    match unlocked.query_map("SELECT * FROM `items_data`", |(id, name, brand, description, group_type, price, image_path, available_quantity)| {
        SqlStream {
            id: id,
            name: name,
            brand: brand,
            description: description,
            group_type: group_type,
            price: price,
            image_path: image_path,
            available_quantity: available_quantity
        }
    }, ) {
        Ok(vector) => {
            Ok(json(&vector))
        }
        Err(e) => {
            Ok(json(&Message {reply : e.to_string()}))
        }
    }
}

pub async fn get_concrete_items_catalog(value : String, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    let mut unlocked = pool.lock().await;
    match unlocked.query_map("SELECT group_type FROM items_data", |(group_type)| {
        ToCompare{ compared: group_type }
    },
    ) {
        Ok(vec) => {
            for elements in vec {
                if value == elements.compared {
                    match unlocked.query_map(format!(r#"SELECT * FROM `items_data` WHERE group_type = "{}""#, value),
                                       |(id, name, brand, description, group_type, price, image_path, available_quantity)| {
                                           SqlStream {
                                               id: id,
                                               name: name,
                                               brand: brand,
                                               description: description,
                                               group_type: group_type,
                                               price: price,
                                               image_path: image_path,
                                               available_quantity: available_quantity
                                           }
                                       }
                    ) {
                        Ok(result) => {return Ok(json(&result))}
                        Err(e) => {return Ok(json(&Message{reply : e.to_string()}))}
                    }
                }
            }
            return Ok(json(&Message{reply : format!("{} - No values found for your request", value)}))
        }
        Err(e) => {
            return Ok(json(&Message{reply : e.to_string()}))
        }
    }
}