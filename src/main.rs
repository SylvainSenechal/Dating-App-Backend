// modules system : https://www.sheshbabu.com/posts/rust-module-system/
mod clients;
mod constants;
mod data_access_layer;
mod my_errors;
mod requests;
mod responses;
mod service_layer;
mod utilities;

use constants::constants::DATABASE_NAME;
// TODO : Rework Actions CI/CD
// TODO : Show when swiping if the user liked me already
// TODO : Stats : How many people fit my criterion I havent swiped yet + How many people are looking for my type
// TODO : Infos bulle (?) qui explique comment l'appli fonctionne, comment les stats fonctionnent
// TODO : rework routing into one liner
// TODO : Lover do not return password
// TODO : Clean struct into Request/Response/DTO folder
// TODO : red dot sur activite swutcher nb new match
// TODO : indicateur horizontal derniere connexion dans message
// TODO : change routes /users/ en /action
// todo : add a report table
// todo : add a suggestions/bugs table / fonctionnalite send developer feedback
// todo : retester les error messages
// todo : check ON DELETE CASCADE
// todo : check enabling foreign key constraint
// todo : voir sse qui spam requetes
// todo : faire un graphe three js ou canvas sur les stats avec des fleches /swipe pas swipe
// todo : LIMIT 1 for query one
use axum::{
    extract::DefaultBodyLimit,
    http,
    http::{HeaderValue, Method, StatusCode},
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::service_layer::sse_service::SseMessage;
use service_layer::user_service::{create_user, delete_user, get_user, update_user};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    connection: Pool<SqliteConnectionManager>,
    txs: Mutex<HashMap<String, broadcast::Sender<SseMessage>>>,
    aws_client: clients::aws::AwsClient,
}

impl AppState {
    async fn new() -> Arc<AppState> {
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

        let aws_client = clients::aws::AwsClient::new().await;
        Arc::new(AppState {
            connection: pool,
            txs: Mutex::new(HashMap::new()),
            aws_client: aws_client,
        })
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "example_tracing_aka_logging=debug,tower_http=debug,axum::rejection=trace".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = AppState::new().await;

    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/:user_uuid", get(get_user))
        .route("/users/:user_uuid", put(update_user))
        .route("/users/:user_uuid", delete(delete_user))
        .route(
            "/users/findlover",
            get(service_layer::user_service::find_lover),
        )
        .route(
            "/users/swipe",
            post(service_layer::user_service::swipe_user),
        )
        .route(
            "/users/:user_uuid/statistics/loved",
            get(service_layer::statistics_service::loved_count),
        )
        .route(
            "/users/:user_uuid/statistics/rejected",
            get(service_layer::statistics_service::rejected_count),
        )
        .route(
            "/users/:user_uuid/statistics/loving",
            get(service_layer::statistics_service::loving_count),
        )
        .route(
            "/users/:user_uuid/statistics/rejecting",
            get(service_layer::statistics_service::rejecting_count),
        )
        .route(
            "/users/:user_uuid/statistics/matching_potential",
            get(service_layer::statistics_service::matching_potential),
        )
        .route(
            "/users/:user_uuid/statistics/traces",
            get(service_layer::statistics_service::backend_activity),
        )
        .route(
            "/messages",
            post(service_layer::message_service::create_message),
        )
        .route(
            "/messages/tick_messages",
            put(service_layer::message_service::green_tick_messages),
        )
        .route(
            "/messages/:love_uuid",
            get(service_layer::message_service::get_love_messages),
        )
        .route(
            "/messages/users/:user_uuid",
            get(service_layer::message_service::get_lover_messages),
        )
        .route("/photos", post(service_layer::photos_service::save_photo))
        .route(
            "/photos/:photo_uuid",
            delete(service_layer::photos_service::delete_photo),
        )
        .route(
            "/photos/switch_photos",
            post(service_layer::photos_service::switch_photos),
        )
        .route(
            "/lovers/:user_uuid",
            get(service_layer::lover_service::get_lovers),
        )
        .route(
            "/lovers/action/:love_uuid/tick_love",
            put(service_layer::lover_service::tick_love),
        )
        .route(
            "/feedbacks",
            post(service_layer::feedback_service::create_feedback),
        )
        .route("/auth", post(service_layer::auth_service::login))
        .route(
            "/auth/refresh",
            post(service_layer::auth_service::token_refresh),
        )
        .route(
            "/server_side_event/:user_private_uuid",
            get(service_layer::sse_service::server_side_event_handler),
        )
        .fallback(p404)
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            service_layer::trace_service::record_trace,
        ))
        .layer(DefaultBodyLimit::max(3 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                // .allow_origin(Any)
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_headers(vec![
                    http::header::CONTENT_TYPE,
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::HeaderName::from_lowercase(b"trace").unwrap(),
                ])
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ]),
        )
        .with_state(app_state);

    let ip: [u8; 4] = [127, 0, 0, 1];
    // let addr = SocketAddr::from(([0, 0, 0, 0], 80));
    let addr = SocketAddr::from((ip, 8080));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn p404() -> (StatusCode, Json<String>) {
    (
        StatusCode::NOT_FOUND,
        Json("Four O Four : Nothing to see here dud 👀".to_string()),
    )
}
