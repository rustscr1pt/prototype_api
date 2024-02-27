use std::sync::{Arc};
use mysql::PooledConn;
use tokio::sync::Mutex;
use warp::Filter;
use crate::mysql_model::establish_connection;
use crate::warp_handlers::{get_concrete_items_catalog, refuse_connection};

mod mysql_model;
mod warp_handlers;
mod warp_injectors;
mod data_models;

#[tokio::main]
async fn main() {
    let connection : Arc<Mutex<PooledConn>> = Arc::new(Mutex::new(establish_connection())); // Shared Pool for working with MySQL at different threads

    let refuse_connection = warp::any().and(warp::method()).and_then(refuse_connection); // Refuse the connection if it doesn't match any filters

    let get_all_positions_catalog = warp::path!("catalog" / "all") // Get all available positions + total amount of items + available categories
        .and(warp::get())
        .and(warp_injectors::with_pool(Arc::clone(&connection)))
        .and_then(warp_handlers::get_all_items_catalog);

    let concrete_positions_catalog = warp::path!("catalog" / String) // Get a list of items for the desired category
        .map(|value : String| {
            value.clone()
        })
        .and(warp::get())
        .and(warp_injectors::with_pool(Arc::clone(&connection)))
        .and_then(get_concrete_items_catalog);

    println!("Server is initialized.\nDeployment address : http://localhost:8000/");

    warp::serve(get_all_positions_catalog.or(concrete_positions_catalog).or(refuse_connection)).run(([0, 0, 0, 0], 8000)).await;
}