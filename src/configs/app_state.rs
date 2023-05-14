use crate::clients;
use crate::configs::config::Config;
use crate::constants::constants::DATABASE_NAME;
use crate::service_layer::sse_service::SseMessage;
use r2d2::Pool;

use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

pub struct AppState {
    pub connection: Pool<SqliteConnectionManager>,
    pub txs: Mutex<HashMap<String, broadcast::Sender<SseMessage>>>,
    pub aws_client: clients::aws::AwsClient,
    pub key_jwt: String,
    pub refresh_key_jwt: String,
}

impl AppState {
    pub async fn new(config: &Config) -> Arc<AppState> {
        let manager = SqliteConnectionManager::file(DATABASE_NAME);
        let pool = r2d2::Pool::builder()
            .max_size(100)
            .build(manager)
            .expect("couldn't create pool");

        let connection = pool.get().unwrap();
        let pragma1 = connection
            .query_row("PRAGMA journal_mode = WAL;", [], |row| {
                let res: String = row.get(0).unwrap();
                Ok(res)
            })
            .expect("Error pragma WAL mode on");
        let pragma2 = connection
            .execute("PRAGMA synchronous = 0;", [])
            .expect("Error pragma synchronous = 0");
        let pragma3 = connection
            .execute("PRAGMA cache_size = 1000000;", [])
            .expect("Error pragma cache_size set");
        // let pragma4 = connection
        //     .execute("PRAGMA foreign_keys = ON;", [])
        //     .expect("Error pragma foreign keys = On");
        // let pragma4 = connection.execute("PRAGMA mmap_size = 30000000000;", []);//.expect("err pragma 3");
        // let pragma5 = connection.execute("PRAGMA locking_mode = NORMAL;", []);//.expect("err pragma 4");

        println!("pragma 1 {:?}", pragma1);
        println!("pragma 2 {:?}", pragma2);
        println!("pragma 3 {:?}", pragma3);
        // println!("pragma 4 {:?}", pragma4);
        let aws_client = clients::aws::AwsClient::new(
            config.r2_account_id.clone(),
            config.r2_image_domain.clone(),
            config.bucket_name.clone(),
        )
        .await;
        Arc::new(AppState {
            connection: pool,
            txs: Mutex::new(HashMap::new()),
            aws_client: aws_client,
            key_jwt: config.key_jwt.clone(),
            refresh_key_jwt: config.refresh_key_jwt.clone(),
        })
    }
}
