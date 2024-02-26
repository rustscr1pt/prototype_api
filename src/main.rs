use std::sync::{Arc};
use mysql::PooledConn;
use tokio::sync::Mutex;
use warp::Filter;
use crate::mysql_model::establish_connection;

mod mysql_model;
mod warp_handlers;
mod warp_injectors;
mod data_models;

#[tokio::main]
async fn main() {
    let connection : Arc<Mutex<PooledConn>> = Arc::new(Mutex::new(establish_connection()));

    let get_all_positions_catalog = warp::path!("catalog" / "all")
        .and(warp::get())
        .and(warp_injectors::with_pool(Arc::clone(&connection)))
        .and_then(warp_handlers::get_all_items_catalog);

    println!("Server is initialized.\nDeployment address : http://localhost:8000/");

    warp::serve(get_all_positions_catalog).run(([0, 0, 0, 0], 8000)).await;
}