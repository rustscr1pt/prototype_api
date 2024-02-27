use std::sync::Arc;
use itertools::Itertools;
use mysql::{PooledConn};
use mysql::prelude::Queryable;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};
use warp::http::Method;
use warp::reply::json;
use crate::data_models::{CatalogMainRequest, Message, SqlStream, ToCompare};

type WebResult<T> = Result<T, Rejection>;


pub async fn refuse_connection(_ : Method) -> WebResult<impl Reply> { // Refuse connection if it doesn't match any filters
    return Ok(json(&Message {
        reply: "This request is forbidden, connection is being dropped".to_string(),
    }))
}

pub async fn get_all_items_catalog(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    let mut unlocked = pool.lock().await;
    match unlocked.query_map("SELECT * FROM `items_data`", |(id, name, brand, description, group_type, price, image_path, available_quantity)| {
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
    }, ) {
        Ok(vector) => {
            Ok(json(&CatalogMainRequest {
                total_items: vector.len() as u16,
                list_of_groups: vector.iter().map(|value| value.group_type.to_string()).collect::<Vec<String>>().into_iter().unique().collect::<Vec<String>>(),
                all_items: vector,
            }))
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