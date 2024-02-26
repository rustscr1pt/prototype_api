use std::fs;
use mysql::{Pool, PooledConn};

pub fn establish_connection() -> PooledConn {
    let pool = Pool::new(fs::read_to_string(r#"C:\Users\User\Desktop\mysql.txt"#).unwrap().trim()).expect("Couldn't connect to a base");
    println!("Connection with MySQL pool is established!");
    return pool.get_conn().unwrap();
}