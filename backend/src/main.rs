use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_web::http::header;
use rusqlite::{Connection};
use actix_cors::Cors;

// mod auth;
mod data_access_layer;
mod service_layer;
mod my_errors;

// TODO : benchmark auth login avec vrai pass pour voir si async utile
// TODO : see and_then()
// TODO : Check swag generation
// modules system : https://www.sheshbabu.com/posts/rust-module-system/
const DATABASE_NAME: &str = "love.db";

pub struct AppState {
    connection: Connection,
}

impl AppState {
    fn new() -> AppState {
        let connection =
            Connection::open(DATABASE_NAME).expect("Could not connect to the database");
        AppState {
            connection: connection,
        }
    }

    fn create_database(&self) {
        println!("Creating user database");
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS users (
                person_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                pseudo TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                age INTEGER
            )",
            // email TEXT NOT NULL UNIQUE,
            [],
            )
            .expect("Could not create table persons");
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
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
        .allowed_header(header::CONTENT_TYPE)
        .max_age(3600);
        App::new()
            .wrap(cors)
            .data(AppState::new())
            .data(web::JsonConfig::default().error_handler(|err, _req| {
                let e = format!("{:?}", err);
                actix_web::error::InternalError::from_response(
                    err,
                    HttpResponse::Conflict().body(e),
                )
                .into()
            }))
            .wrap(middleware::Logger::default())
            .service(
                web::scope("/users")
                    .service(
                        web::resource("")
                            .route(web::get().to(service_layer::user_service::get_users))
                            .route(web::post().to(service_layer::user_service::create_user)),
                    )
                    .service(
                        web::resource("/{user_id}")
                            .route(web::get().to(service_layer::user_service::get_user)), 
                            // .route(web::delete().to(delete_user))
                            // .route(web::patch().to(update_user))
                    ),
            )
            .service(
                web::scope("/auth")
                .service(
                    web::resource("")
                    .route(web::post().to(service_layer::auth_service::login))
                )
                .service(
                    web::resource("/refresh")
                    .route(web::post().to(service_layer::auth_service::token_refresh))
                )
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

    // let p = Person{age: 25, pseudo: "sylvain".to_string(), ..Default::default()};
    // Person::create_person(&app.connection,p)?;
    // Person::get_persons(&app.connection)?;
    // match Person::get_person(&app.connection, "hugokjhui") {
    //     Ok(val) => println!("on : {:?}", val),
    //     Err(e) => println!("nope : {:?}", e)
    // }
    // println!("getting one person : {:?}", Person::get_person(&app.connection, "hugo"));
    // let a = Person{..Default::default()};
    // println!("getting one  : {:?}", a);

    // Ok(())
}
