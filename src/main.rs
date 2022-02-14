use actix::Actor;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use rusqlite::Connection;
use std::collections::HashMap;

// modules system : https://www.sheshbabu.com/posts/rust-module-system/
mod constants;
mod data_access_layer;
mod my_errors;
mod service_layer;

use constants::constants::DATABASE_NAME;
// TODO : Database creation outside
// TODO : Rework Actions CI/CD
// TODO : Randomize the potential lover selection

#[derive(Debug)]
pub struct AppState {
    connection: Connection,
}

impl AppState {
    fn new() -> AppState {
        let connection =
        Connection::open(DATABASE_NAME).expect("Could not connect to the database");
        let pragma1 = connection.query_row("PRAGMA journal_mode = WAL;", [], |row| {
            let res: String = row.get(0).unwrap();
            Ok(res)
        }).expect("Error pragma WAL mode on");
        let pragma2 = connection.execute("PRAGMA synchronous = 0;", []).expect("Error pragma synchronous = 0");
        let pragma3 = connection.execute("PRAGMA cache_size = 1000000;", []).expect("Error pragma cache_size set");
        let pragma4 = connection.execute("PRAGMA foreign_keys = ON;", []).expect("Error pragma foreign keys = On");
        // let pragma4 = connection.execute("PRAGMA mmap_size = 30000000000;", []);//.expect("err pragma 3");
        // let pragma5 = connection.execute("PRAGMA locking_mode = NORMAL;", []);//.expect("err pragma 4");

        println!("pragma 1 {:?}", pragma1);
        println!("pragma 2 {:?}", pragma2);
        println!("pragma 3 {:?}", pragma3);
        println!("pragma 4 {:?}", pragma4);

        AppState {
            connection: connection,
        }
    }

    fn create_database(&self) {
        println!("Creating user database");
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS Users (
                user_id             INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                name                TEXT NOT NULL,
                password            TEXT NOT NULL,
                email               TEXT NOT NULL UNIQUE,
                age                 INTEGER CHECK (age > 17 AND age < 128) NOT NULL,
                latitude            REAL NOT NULL,
                longitude           REAL NOT NULL,
                gender              TEXT CHECK (gender IN ('male','female')) NOT NULL,
                looking_for         TEXT CHECK (looking_for IN ('male','female')) NOT NULL,
                search_radius       INTEGER CHECK (search_radius > 0 AND search_radius < 65535) NOT NULL DEFAULT 10, --unit is kilometers
                looking_for_age_min INTEGER CHECK (looking_for_age_min > 17 AND looking_for_age_min < 128 AND looking_for_age_min <= looking_for_age_max) NOT NULL DEFAULT 18,
                looking_for_age_max INTEGER CHECK (looking_for_age_max > 17 AND looking_for_age_max < 128) NOT NULL DEFAULT 127,
                description         TEXT CHECK(LENGTH(message) <= 1000) DEFAULT ''
            )",
                [],
            )
            .expect("Could not create table Users");
        self.connection
            .execute("CREATE INDEX IF NOT EXISTS nomIndex ON Users(name)", [])
            .expect("Could not create index on table persons");

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS Photos (
                photo_id    INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                user_id     INTEGER NOT NULL,
                url         TEXT NOT NULL,
                FOREIGN KEY(user_id) REFERENCES Users(user_id)
            )",
                [],
            )
            .expect("Could not create table photos");

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS MatchingResults (
                match_id            INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                swiper              INTEGER NOT NULL,
                swiped              INTEGER NOT NULL,
                love                INTEGER CHECK (love IN (0, 1)) NOT NULL,
                FOREIGN KEY(swiper) REFERENCES Users(user_id),
                FOREIGN KEY(swiped) REFERENCES Users(user_id),
                UNIQUE (swiper, swiped)
            )",
                [],
            )
            .expect("Could not create table MatchingResults");

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS Lovers (
                love_id             INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                lover1              INTEGER NOT NULL,
                lover2              INTEGER NOT NULL,
                FOREIGN KEY(lover1) REFERENCES Users(user_id),
                FOREIGN KEY(lover2) REFERENCES Users(user_id),
                UNIQUE (lover1, lover2)
            )",
                [],
            )
            .expect("Could not create table Lovers");

        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS Messages (
                message_id  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                message     TEXT CHECK(LENGTH(message) <= 1000),
                poster_id   INTEGER NOT NULL,
                love_id     INTEGER NOT NULL,
                date        TEXT NOT NULL,  --UTC ISO8601 from Rust Crate=chrono, example : 2022-02-14T19:47:51.028632Z
                FOREIGN KEY(poster_id) REFERENCES Users(user_id),
                FOREIGN KEY(love_id)   REFERENCES Lovers(love_id)
            )",
                [],
            )
            .expect("Could not create table Messages");
    }
}

async fn p404() -> HttpResponse {
    HttpResponse::NotFound().body("Four O Four : Nothing to see here dud 👀")
}

async fn fake_admin() -> HttpResponse {
    println!("Fake admin visited");
    HttpResponse::NotFound().body("What are you doing here 👀")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var(
        "RUST_LOG",
        "actix_web=debug,actix_server=info,actix_web=info",
    );
    env_logger::init();

    let app: AppState = AppState::new();
    app.create_database();

    let server = service_layer::websocket_service::Server {
        sessions: HashMap::new(),
        love_chat_rooms: HashMap::new(),
    }
    .start();

    // TODO : Cors ELI5
    // TODO : logger middleware
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        App::new()
            .wrap(cors)
            .data(AppState::new())
            .data(server.clone())
            .data(web::JsonConfig::default().error_handler(|err, _req| {
                let e = format!("{:?}", err);
                println!("conflit");
                println!("conflit {}", e);
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::Conflict().json(e),
                )
                .into()
            }))
            .wrap(middleware::Logger::default())
            .route(
                "/ws/",
                web::get().to(service_layer::websocket_service::index_websocket),
            )
            .route(
                "/admin",
                web::to(fake_admin),
            )
            .service(
                web::scope("/users")
                    .service(
                        web::resource("")
                            .route(web::post().to(service_layer::user_service::create_user))
                            // .route(web::get().to(service_layer::user_service::get_users)), dangerous route..
                    )
                    .service(
                        web::resource("/{user_id}")
                            .route(web::get().to(service_layer::user_service::get_user))
                            .route(web::put().to(service_layer::user_service::update_user)),
                    )
                    .service(
                        web::resource("/{user_id}/findlover")
                            .route(web::get().to(service_layer::user_service::find_love)),
                    )
                    .service(
                        web::resource("/{swiper_id}/loves/{swiped_id}")
                            .route(web::post().to(service_layer::user_service::swipe_user)),
                    )
                    .service(
                        web::resource("/{user_id}/statistics/loved")
                            .route(web::get().to(service_layer::statistics_service::loved_count)),
                    )
                    .service(
                        web::resource("/{user_id}/statistics/rejected").route(
                            web::get().to(service_layer::statistics_service::rejected_count),
                        ),
                    )
                    .service(
                        web::resource("/{user_id}/statistics/loving")
                            .route(web::get().to(service_layer::statistics_service::loving_count)),
                    )
                    .service(
                        web::resource("/{user_id}/statistics/rejecting").route(
                            web::get().to(service_layer::statistics_service::rejecting_count),
                        ),
                    ),
            )
            .service(
                web::scope("/messages")
                    .service(
                        web::resource("")
                            .route(web::post().to(service_layer::message_service::create_message)),
                    )
                    .service(
                        web::resource("/{love_id}").route(
                            web::get().to(service_layer::message_service::get_love_messages),
                        ),
                    )
                    .service(
                        web::resource("/users/{user_id}").route(
                            web::get().to(service_layer::message_service::get_lover_messages),
                        ),
                    ),
            )
            .service(
                web::resource("/photos")
                    .route(web::post().to(service_layer::photos_service::save_file)),
            )
            .service(
                web::resource("/lovers/{user_id}")
                    .route(web::get().to(service_layer::lover_service::get_lovers)),
            )
            .service(
                web::scope("/auth")
                    .service(
                        web::resource("").route(web::post().to(service_layer::auth_service::login)),
                    )
                    .service(
                        web::resource("/refresh")
                            .route(web::post().to(service_layer::auth_service::token_refresh)),
                    ),
            )
            .default_service(web::to(p404))
    })
    .bind("127.0.0.1:8080")?
    // .workers(8) // Default is number of physical cores
    .run()
    .await
}
