// modules system : https://www.sheshbabu.com/posts/rust-module-system/
mod clients;
mod configs;
mod constants;
mod data_access_layer;
mod my_errors;
mod requests;
mod responses;
mod service_layer;
mod utilities;

// TODO : Rework Actions CI/CD
// TODO : Show when swiping if the user liked me already
// TODO : Stats : How many people fit my criterion I havent swiped yet + How many people are looking for my type
// TODO : Infos bulle (?) qui explique comment l'appli fonctionne, comment les stats fonctionnent
// TODO : red dot sur activite swutcher nb new match
// TODO : indicateur horizontal derniere connexion dans message
// TODO : change routes /users/ en /action
// todo : add a report table
// todo : retester les error messages
// todo : check ON DELETE CASCADE
// todo : check enabling foreign key constraint
// todo : voir sse qui spam requetes
// todo : faire un graphe three js ou canvas sur les stats avec des fleches /swipe pas swipe
// todo : check tokio tower trace
// todo : rework les notifs
use axum::{
    extract::DefaultBodyLimit,
    http,
    http::{HeaderValue, Method, StatusCode},
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    let config = configs::config::Config::new();
    let app_state = configs::app_state::AppState::new(&config).await;
    println!("config : {:?}", config);

    let app = Router::new()
        .route("/users", post(service_layer::user_service::create_user))
        .route(
            "/users/:user_uuid",
            get(service_layer::user_service::get_user),
        )
        .route(
            "/users/:user_uuid",
            put(service_layer::user_service::update_user),
        )
        .route(
            "/users/:user_uuid",
            delete(service_layer::user_service::delete_user),
        )
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
                .allow_origin(
                    config
                        .wed_domains
                        .iter()
                        .map(|domain| {
                            domain
                                .parse::<HeaderValue>()
                                .expect("parse web domains into HeaderValue failed")
                        })
                        .collect::<Vec<HeaderValue>>(),
                )
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
        .with_state(app_state.clone());

    let addr = SocketAddr::from((config.ip, config.port));
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
