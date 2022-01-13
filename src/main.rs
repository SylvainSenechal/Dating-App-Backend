use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use rusqlite::Connection;

// mod auth;
mod constants;
mod data_access_layer;
mod my_errors;
mod service_layer;

use constants::constants::DATABASE_NAME;
// TODO : see and_then()
// TODO : Check swag generation
// modules system : https://www.sheshbabu.com/posts/rust-module-system/

pub struct AppState {
    connection: Connection,
}

impl AppState {
    fn new() -> AppState {
        let connection =
            Connection::open(DATABASE_NAME).expect("Could not connect to the database");
        let res1 = connection.query_row("PRAGMA journal_mode = WAL;", [], |row| {
            let res: String = row.get(0).unwrap();
            Ok(res)
        }); //.expect("err pragma 1");
        let res2 = connection.execute("PRAGMA synchronous = 0;", []); //.expect("err pragma 2");
        let res3 = connection.execute("PRAGMA cache_size = 1000000;", []); //.expect("err pragma 3");
                                                                           // let _ = connection.execute("PRAGMA mmap_size = 30000000000;", []);//.expect("err pragma 3");
                                                                           // let res4 = connection.execute("PRAGMA locking_mode = NORMAL;", []);//.expect("err pragma 4");

        println!("1 {:?}", res1);
        println!("2 {:?}", res2);
        println!("3 {:?}", res3);
        // println!("4 {:?}", res4);
        AppState {
            connection: connection,
        }
    }

    fn create_database(&self) {
        println!("Creating user database");
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS users (
                person_id       INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                pseudo          TEXT NOT NULL,
                password        TEXT NOT NULL,
                email           TEXT NOT NULL UNIQUE,
                age             INTEGER CHECK (age > 0 AND age < 128) NOT NULL,
                latitude        REAL NOT NULL,
                longitude       REAL NOT NULL,
                gender          TEXT CHECK (gender IN ('male','female')) NOT NULL,
                looking_for     TEXT CHECK (looking_for IN ('male','female')) NOT NULL,
                search_radius   INTEGER CHECK (search_radius > 0 AND search_radius < 65535) NOT NULL DEFAULT 10 --unit is kilometers
            )",
                [],
            )
            .expect("Could not create table persons");
        self.connection
            .execute("CREATE INDEX IF NOT EXISTS nomIndex ON users(pseudo)", [])
            .expect("Could not create index on table persons");
    }
}

async fn p404() -> HttpResponse {
    HttpResponse::NotFound().body("Four O Four : Nothing to see here dud 👀")
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

    // TODO : Cors ELI5
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "POST", "PUT"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

            App::new()
            .wrap(cors)
            .data(AppState::new())
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
            .service(
                web::resource("/users")
                    .route(web::post().to(service_layer::user_service::create_user))
                    .route(web::get().to(service_layer::user_service::get_users)),
            )
            .service(
                web::resource("/users/{user_id}")
                    .route(web::get().to(service_layer::user_service::get_user))
                    .route(web::put().to(service_layer::user_service::update_user)),
            )
            .service(
                web::resource("/photos")
                    .route(web::post().to(service_layer::photos_service::save_file)),
            )
            .service(
                web::resource("/auth").route(web::post().to(service_layer::auth_service::login)),
            )
            .service(
                web::resource("/auth/refresh")
                    .route(web::post().to(service_layer::auth_service::token_refresh)),
            )
            .default_service(
                web::resource("")
                    // TODO : revoir ca
                    .route(web::get().to(p404))
                    .route(
                        web::route()
                            // .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
