use std::fmt::{Debug, Display};
use std::sync::Arc;
use mysql::{PooledConn};
use rand::Rng;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use warp::{Rejection, Reply, reply};
use warp::http::Method;
use warp::reply::{json, Json};
use crate::data_models::{CatalogMainRequest, CategoryMainRequest, ConcreteItemLayout, IndexBasicRequest, Message, SqlStream};
use crate::mysql_model::{all_from_table_where_group_type, pick_3_random_recommendations, remove_repeating_elements_to_string, select_all_from_table, select_from_table_by_id, select_group_type_from_table};

type WebResult<T> = Result<T, Rejection>;


pub async fn refuse_connection(_ : Method) -> WebResult<impl Reply> { // Refuse connection if it doesn't match any filters
    Ok(reply::with_header(json(&Message { reply: "This request is forbidden, connection is dropped".to_string()}), "Access-Control-Allow-Origin", "*"))
}

fn reply_error<T>(error : T) -> WebResult<reply::WithHeader<Json>> // Reply with error.
    where T : Display
{
    Ok(reply::with_header(json(&Message {reply : error.to_string()}), "Access-Control-Allow-Origin", "*"))
}

pub async fn get_all_items_catalog(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> { // Get all items from the tables catalog.
    let mut unlocked = pool.lock().await;
    match select_all_from_table(&mut unlocked) {
        Ok(vector) => {
            Ok(reply::with_header(json(&CatalogMainRequest {
                total_items: vector.len() as u16,
                list_of_groups: remove_repeating_elements_to_string(&vector),
                all_items: vector,
            }), "Access-Control-Allow-Origin", "*"))
        }
        Err(e) => {
            reply_error(e)
        }
    }
}

pub async fn get_concrete_items_catalog(value : String, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> { // Value is passed in, and filtered catalog is returned
    let mut unlocked = pool.lock().await;
    if value == "all" {
        match select_all_from_table(&mut unlocked) {
            Ok(vec) => {
                Ok(reply::with_header(json(&vec), "Access-Control-Allow-Origin", "*"))
            }
            Err(e) => {reply_error(e)}
        }
    }
    else {
        match select_group_type_from_table(&mut unlocked) {
            Ok(vec) => {
                for elements in vec {
                    if value == elements.compared {
                        match all_from_table_where_group_type(&mut unlocked, value) {
                            Ok(result) => {return Ok(reply::with_header(json(&result), "Access-Control-Allow-Origin", "*"))}
                            Err(e) => {return reply_error(e)}
                        }
                    }
                }
                return Ok(reply::with_header(json(&Message{reply : format!("{} - No values found for your request", value)}), "Access-Control-Allow-Origin", "*"))
            }
            Err(e) => {
                reply_error(e)
            }
        }
    }
}

pub fn items_async_counter(element : String, sqlstream_cloned : Arc<Vec<SqlStream>>, release_cloned : Arc<Mutex<Vec<CategoryMainRequest>>>) -> JoinHandle<()> { // Count amount for every element in async mode. Element is passed outside from filtered function.
    return tokio::spawn(async move {
        let mut counter : u16 = 0;
        for categories in sqlstream_cloned.iter() {
            if element == categories.group_type {
                counter += categories.available_quantity as u16
            }
        }
        let mut locked_release = release_cloned.lock().await;
        locked_release.push(CategoryMainRequest {
            category: element,
            amount: counter,
        });
        drop(locked_release);
    })
}

pub async fn main_screen_getter(pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> { // Two stages. 1) We get 6 random elements to display. 2) We get available categories and count the amount of items for each of it.
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
            for element in filtered_categories { // We get a vec of cleaned and not repeating elements and then count amount of items inside it.
                let active_count_thread = items_async_counter(element, Arc::clone(&arc_vector), Arc::clone(&release_structs)); // We pass in an element and then count amount of items inside.
                active_threads_holder.push(active_count_thread); // We push a thread inside the await pool.
            }
            futures::future::join_all(active_threads_holder).await; // We wait for every thread in a pool to finish its job.
            let unlocked_final = release_structs.lock().await;
            Ok(reply::with_header(json(&IndexBasicRequest {
                random_positions: random_vector,
                available_categories: unlocked_final.clone(),
            }), "Access-Control-Allow-Origin", "*"))
        }
        Err(e) => {
            reply_error(e)
        }
    }
}

pub async fn get_item_by_id(id : String, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
    let mut unlocked = pool.lock().await;
    match id.parse::<u16>() {
        Ok(num) => {
            match select_from_table_by_id(&mut unlocked, num) {
                Ok(value) => { // got element by its id
                    if value.len() == 0 {
                        Ok(reply::with_header(json(&Message {reply : "No items found for this ID. Please try another one.".to_string()}), "Access-Control-Allow-Origin", "*"))
                    }
                    else {
                        match pick_3_random_recommendations(&mut unlocked, num, value[0].group_type.clone()) {
                            Ok(random) => {
                                return Ok(reply::with_header(json(&ConcreteItemLayout {
                                    item: value[0].clone(),
                                    recommendations: random,
                                }), "Access-Control-Allow-Origin", "*"))
                            }
                            Err(e) => {
                                Ok(reply::with_header(json(&Message {reply : e.to_string()}), "Access-Control-Allow-Origin", "*"))
                            }
                        }
                    }
                }
                Err(e) => {
                    reply_error(e)
                }
            }
        }
        Err(e) => {
            reply_error(e)
        }
    }
}

// pub async fn place_an_order_post_request<T : Clone>(order : T, pool : Arc<Mutex<PooledConn>>) -> WebResult<impl Reply> {
//
// }
