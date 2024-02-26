use std::sync::Arc;
use mysql::PooledConn;
use mysql::prelude::Queryable;
use tokio::sync::Mutex;
use warp::{Rejection, Reply};
use warp::reply::json;
use crate::data_models::{Message, SqlStream};

type WebResult<T> = Result<T, Rejection>;

pub async fn get_all_items_catalog(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    let mut unlocked = pool.lock().await;
    match unlocked.query_map("SELECT * FROM `site_data`", |(id, name, image_path, price)| {
        SqlStream {
            id: id,
            name: name,
            image_path: image_path,
            price: price,
        }
    }, ) {
        Ok(vector) => {
            Ok(json(&vector))
        }
        Err(e) => {
            Ok(json(&Message {
                reply: e.to_string(),
            }))
        }
    }
}