use std::sync::Arc;
use mysql::{PooledConn};
use mysql::prelude::Queryable;
use rand::Rng;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use warp::{Rejection, Reply};
use warp::http::Method;
use warp::reply::json;
use crate::data_models::{CatalogMainRequest, CategoryMainRequest, IndexBasicRequest, Message, SqlStream, ToCompare};
use crate::mysql_model::{all_from_table_where_group_type, remove_repeating_elements_to_string, select_all_from_table, select_group_type_from_table};

type WebResult<T> = Result<T, Rejection>;


pub async fn refuse_connection(_ : Method) -> WebResult<impl Reply> { // Refuse connection if it doesn't match any filters
    Ok(warp::reply::with_header(json(&Message { reply: "This request is forbidden, connection is dropped".to_string()}), "Access-Control-Allow-Origin", "*"))
}

pub async fn get_all_items_catalog(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> { // Get all items from the tables catalog.
    let mut unlocked = pool.lock().await;
    match select_all_from_table(&mut unlocked) {
        Ok(vector) => {
            Ok(warp::reply::with_header(json(&CatalogMainRequest {
                total_items: vector.len() as u16,
                list_of_groups: remove_repeating_elements_to_string(&vector),
                all_items: vector,
            }), "Access-Control-Allow-Origin", "*"))
        }
        Err(e) => {
            Ok(warp::reply::with_header(json(&Message {reply : e.to_string()}), "Access-Control-Allow-Origin", "*"))
        }
    }
}

pub async fn get_concrete_items_catalog(value : String, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> { // Value is passed in, and filtered catalog is returned
    let mut unlocked = pool.lock().await;
    if value == "all" {
        match select_all_from_table(&mut unlocked) {
            Ok(vec) => {
                Ok(warp::reply::with_header(json(&vec), "Access-Control-Allow-Origin", "*"))
            }
            Err(e) => {Ok(warp::reply::with_header(json(&Message {reply : e.to_string()}), "Access-Control-Allow-Origin", "*"))}
        }
    }
    else {
        match select_group_type_from_table(&mut unlocked) {
            Ok(vec) => {
                for elements in vec {
                    if value == elements.compared {
                        match all_from_table_where_group_type(&mut unlocked, value) {
                            Ok(result) => {return Ok(warp::reply::with_header(json(&result), "Access-Control-Allow-Origin", "*"))}
                            Err(e) => {return Ok(warp::reply::with_header(json(&Message{reply : e.to_string()}), "Access-Control-Allow-Origin", "*"))}
                        }
                    }
                }
                return Ok(warp::reply::with_header(json(&Message{reply : format!("{} - No values found for your request", value)}), "Access-Control-Allow-Origin", "*"))
            }
            Err(e) => {
                return Ok(warp::reply::with_header(json(&Message{reply : e.to_string()}), "Access-Control-Allow-Origin", "*"))
            }
        }
    }
}

pub async fn main_screen_getter(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    let mut unlocked = pool.lock().await;
    match select_all_from_table(&mut unlocked) {
        Ok(vector) => {
            let mut random_vector : Vec<SqlStream> = Vec::with_capacity(6);
            for _ in 0..6 {
                random_vector.push(vector.clone().get(rand::thread_rng().gen_range(0..vector.len())).unwrap().clone());
            }

            let filtered_categories = remove_repeating_elements_to_string(&vector);
            let mut active_threads_holder : Vec<JoinHandle<()>> = Vec::with_capacity(filtered_categories.len() + 1);
            let release_structs : Arc<Mutex<Vec<CategoryMainRequest>>> = Arc::new(Mutex::new(Vec::with_capacity(filtered_categories.len() + 1)));
            let arc_vector = Arc::new(vector);
            for element in filtered_categories {

                let sqlstream_cloned = Arc::clone(&arc_vector);
                let release_cloned = Arc::clone(&release_structs);

                let active_counter = tokio::spawn(async move {
                    let mut counter : u16 = 0;
                    let mut locked_release = release_cloned.lock().await;

                    for category in sqlstream_cloned.iter() {
                        if element == category.group_type {
                            counter += category.available_quantity as u16
                        }
                    }
                    locked_release.push(CategoryMainRequest {
                        category: element,
                        amount:  counter});
                    drop(locked_release);
                });
                active_threads_holder.push(active_counter)
            }
            futures::future::join_all(active_threads_holder).await;
            let unlocked_final = release_structs.lock().await;
            Ok(warp::reply::with_header(json(&IndexBasicRequest {
                random_positions: random_vector,
                available_categories: unlocked_final.clone(),
            }), "Access-Control-Allow-Origin", "*"))
        }
        Err(e) => {
            Ok(warp::reply::with_header(json(&Message {reply : e.to_string()}), "Access-Control-Allow-Origin", "*"))
        }
    }
}