use rusqlite::{Connection};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};

mod auth;
mod data_access_layer;
mod my_errors;

use auth::coucou;

// TODO : see and_then()

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
        self.connection
            .execute(
                "CREATE TABLE IF NOT EXISTS persons (
                person_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                pseudo TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL UNIQUE,
                age INTEGER
            )",
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
    coucou();
    // std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var(
        "RUST_LOG",
        "actix_web=debug,actix_server=info,actix_web=info",
    );
    env_logger::init();

    let app: AppState = AppState::new();
    app.create_database();

    HttpServer::new(|| {
        App::new()
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
                            .route(web::get().to(data_access_layer::user::User::get_users))
                            .route(web::post().to(data_access_layer::user::User::create_user))
                    )
                    .service(
                        web::resource("/{user_id}").route(web::get().to(data_access_layer::user::User::get_user)), // .route(web::post().to(post_user))
                                                                                    // .route(web::delete().to(delete_user))
                                                                                    // .route(web::patch().to(update_user))
                    ),
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