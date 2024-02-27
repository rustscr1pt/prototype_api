use std::sync::Arc;
use mysql::PooledConn;
use tokio::sync::Mutex;
use warp::Filter;

pub fn with_pool(pool : Arc<Mutex<PooledConn>>) -> impl Filter<Extract = (Arc<Mutex<PooledConn>>,), Error = std::convert::Infallible> + Clone { // inject the Pooled connection inside the filter
    warp::any().map(move ||  pool.clone())
}