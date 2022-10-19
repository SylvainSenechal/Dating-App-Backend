use actix::Actor;
use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{dev::Service, middleware, web, App, HttpResponse, HttpServer};
use data_access_layer::trace_dal::TraceRequest;
use futures_util::future::FutureExt;
use rusqlite::Connection;
use std::collections::HashMap;
use std::usize;

// modules system : https://www.sheshbabu.com/posts/rust-module-system/
mod constants;
mod data_access_layer;
mod my_errors;
mod service_layer;
mod utilities;

use constants::constants::DATABASE_NAME;
// TODO : Rework Actions CI/CD
// TODO : Show last active time on discussion
// TODO : Show when swiping if the user liked me already
// TODO : Stats : How many people fit my criterion I havent swiped yet + How many people are looking for my type
// TODO : Infos bulle (?) qui explique comment l'appli fonctionne, comment les stats fonctionnent
// TODO : Gerer OPTIONS request
// TODO : rework routing into one liner
// TODO : Check sql injections in message, other fields
// TODO : Lover do not return password
#[derive(Debug)]
pub struct AppState {
    connection: Connection,
}

impl AppState {
    fn new() -> AppState {
        let connection =
            Connection::open(DATABASE_NAME).expect("Could not connect to the database");
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
            connection: connection,
        }
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
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .allowed_header("trace")
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap_fn(|req, srv| {
                if let Some(db) = req.app_data::<web::Data<AppState>>() {
                    let mut trace = TraceRequest {
                        trace_id: None::<usize>,
                        ip: req.peer_addr(),
                        method: req.method().as_str(),
                        path: req.path(),
                        query_string: req.query_string(),
                        data: None,
                    };
                    if let Some(trace_id) = req.headers().get("Trace") {
                        let trace_id = trace_id
                            .to_str()
                            .expect("header to str failed")
                            .parse::<usize>()
                            .expect("str to usize failed");
                        trace.trace_id = Some(trace_id);
                    }
                    data_access_layer::trace_dal::create_trace(db, trace)
                        .expect("dal create trace failed")
                }

                srv.call(req).map(|res| res)
            })
            .data(AppState::new())
            .data(server.clone()) // are we having differents independent ws server here ?
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
            .route("/admin", web::to(fake_admin))
            .service(
                web::scope("/users")
                    .service(
                        web::resource("")
                            .route(web::post().to(service_layer::user_service::create_user)), // .route(web::get().to(service_layer::user_service::get_users)), dangerous route..
                    )
                    .service(
                        web::resource("/{user_id}")
                            .route(web::get().to(service_layer::user_service::get_user))
                            .route(web::put().to(service_layer::user_service::update_user))
                            .route(web::delete().to(service_layer::user_service::delete_user)),
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
