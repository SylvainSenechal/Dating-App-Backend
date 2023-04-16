// use data_access_layer::trace_dal::TraceRequest;

// modules system : https://www.sheshbabu.com/posts/rust-module-system/
mod constants;
mod data_access_layer;
mod my_errors;
mod service_layer;
mod utilities;

use constants::constants::DATABASE_NAME;
// TODO : Rework Actions CI/CD
// TODO : Show when swiping if the user liked me already
// TODO : Stats : How many people fit my criterion I havent swiped yet + How many people are looking for my type
// TODO : Infos bulle (?) qui explique comment l'appli fonctionne, comment les stats fonctionnent
// TODO : Gerer OPTIONS request
// TODO : rework routing into one liner
// TODO : Lover do not return password
// TODO : Clean struct into Request/Response/DTO folder
// TODO : red dot sur activite swutcher nb new match
// TODO : indicateur horizontal derniere connexion dans message
// TODO : Local cache
// TODO : get les ocnnections une fois par fonction..
// TODO : change routes /users/ en /action
// todo : refactor id => uuid
// todo : add a report table
// todo : add a suggestions/bugs table / fonctionnalite send developer feedback
// todo : retester les error messages
// todo : check ON DELETE CASCADE
// TODO : handle sse connection closed

// async fn fake_admin() -> HttpResponse {
//     println!("Fake admin visited");
//     HttpResponse::NotFound().body("What are you doing here 👀")
// }

use axum::{
    http,
    http::{HeaderValue, Method, StatusCode},
    routing::{delete, get, post, put},
    Json, Router,
};

use r2d2::Pool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
extern crate r2d2;
extern crate r2d2_sqlite; // todo check "extern" keyword
use r2d2_sqlite::SqliteConnectionManager;

use crate::service_layer::sse_service::SseMessage;
use service_layer::user_service::{create_user, delete_user, get_user, update_user};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct AppState {
    connection: Pool<SqliteConnectionManager>,
    txs: Mutex<HashMap<String, broadcast::Sender<SseMessage>>>, // TODO : revoir user broadcast, or oneshoot etc ?
}

impl AppState {
    fn new() -> AppState {
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
        let pragma4 = connection
            .execute("PRAGMA foreign_keys = ON;", [])
            .expect("Error pragma foreign keys = On");
        // let pragma4 = connection.execute("PRAGMA mmap_size = 30000000000;", []);//.expect("err pragma 3");
        // let pragma5 = connection.execute("PRAGMA locking_mode = NORMAL;", []);//.expect("err pragma 4");

        println!("pragma 1 {:?}", pragma1);
        println!("pragma 2 {:?}", pragma2);
        println!("pragma 3 {:?}", pragma3);
        println!("pragma 4 {:?}", pragma4);

        AppState {
            connection: pool,
            txs: Mutex::new(HashMap::new()),
        }
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
        .route("/photos", post(service_layer::photos_service::save_file))
        .route(
            "/lovers/:user_uuid",
            get(service_layer::lover_service::get_lovers),
        )
        .route(
            "/lovers/action/:love_uuid/tick_love",
            put(service_layer::lover_service::tick_love),
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
        .with_state(Arc::new(AppState::new()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
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

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     // std::env::set_var("RUST_LOG", "actix_web=info");
//     std::env::set_var(
//         "RUST_LOG",
//         "actix_web=debug,actix_server=info,actix_web=info",
//     );
//     // env_logger::init();

//     let server = service_layer::websocket_service::Server {
//         sessions: HashMap::new(),
//         love_chat_rooms: HashMap::new(),
//     }
//     .start();

//     HttpServer::new(move || {
//         let cors = Cors::default()
//             .allowed_origin("http://localhost:3000")
//             .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
//             .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
//             .allowed_header(header::CONTENT_TYPE)
//             .allowed_header("trace")
//             .max_age(3600);

//         App::new()
//             .wrap(cors)
//             .wrap_fn(|req, srv| {
//                 if let Some(db) = req.app_data::<web::Data<AppState>>() {
//                     let mut trace = TraceRequest {
//                         trace_id: None::<usize>,
//                         ip: req.peer_addr(),
//                         method: req.method().as_str(),
//                         path: req.path(),
//                         query_string: req.query_string(),
//                         data: None,
//                     };
//                     if let Some(trace_id) = req.headers().get("Trace") {
//                         let trace_id = trace_id
//                             .to_str()
//                             .expect("header to str failed")
//                             .parse::<usize>()
//                             .expect("str to usize failed");
//                         trace.trace_id = Some(trace_id);
//                     }
//                     data_access_layer::trace_dal::create_trace(db, trace)
//                         .expect("dal create trace failed")
//                 }

//                 srv.call(req).map(|res| res)
//             })
