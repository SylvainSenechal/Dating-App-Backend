use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result as actixResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use argon2::{
    Algorithm, Argon2, Error, Params, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version, password_hash::SaltString,
};
use rand::Rng; // 0.8.0

mod auth;
mod data_access_layer;
mod my_errors;

use auth::coucou;

// TODO : see and_then()
// modules system
// https://www.sheshbabu.com/posts/rust-module-system/
const DATABASE_NAME: &str = "love.db";
const M_COST: u32 = 15_000;// m_cost is the memory size, expressed in kilobytes
const T_COST: u32 = 4; // t_cost is the number of iterations;
const P_COST: u32 = 1; //p_cost is the degree of parallelism.
const OUTPUT_LEN: usize = 32; // determines the length of the returned hash in bytes

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
                "CREATE TABLE IF NOT EXISTS persons (
                person_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
                pseudo TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                age INTEGER
            )",
                [],
            )
            .expect("Could not create table persons");
    }
}

use crate::my_errors::sqlite_errors::map_sqlite_error;
use crate::my_errors::sqlite_errors::SqliteError;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateUser {
    #[serde(default)]
    pseudo: String,
    #[serde(default)]
    email: String,
    #[serde(default)]
    age: u8,
}

async fn create_user(
    db: web::Data<AppState>,
    create_user_request: web::Json<CreateUser>,
) -> actixResult<HttpResponse, SqliteError> {

    let user = data_access_layer::user::User::get_user(&db, create_user_request.pseudo.to_string()).await;
    println!("eeeee : {:?}", user);

    let hasher: Argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(M_COST, T_COST, P_COST, Some(OUTPUT_LEN))
            .expect("Failed to build params for Argon2id") // TODO : clean error
    );
    
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hashed_password = hasher.hash_password("nulPass".as_bytes(), &salt)
        .expect("Could not hash password"); // TODO : clean error

    let phc_string = hashed_password.to_string();
    // println!("hash strinnnnggg {}", hashed_password);
    // println!("hash strinnnnggg {}", phc_string);
    // let rehash = PasswordHash::new(&phc_string).unwrap();

    // Argon2::default().verify_password("nulPass".as_bytes(), &hashed_password).expect("could not verify");
    // Argon2::default().verify_password("nulPass".as_bytes(), &rehash).expect("could not verify");

    match user {
        Ok(_) => println!("found user"),
        Err(SqliteError::NotFound) => {
            println!("user not found");
            let user = data_access_layer::user::User{
                pseudo: create_user_request.pseudo.to_string(),
                email: create_user_request.email.to_string(),
                password: phc_string,
                age: Some(create_user_request.age)
            };
            data_access_layer::user::User::create_user(&db, user).await?;
        },
        _ => println!("Sqlite error"),
    }

  




    Ok(HttpResponse::Ok().body("User created"))
}

async fn p404() -> HttpResponse {
    HttpResponse::NotFound().body("Four O Four : Nothing to see here dud 👀")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // coucou();
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
                            .route(web::post().to(create_user)),
                    )
                    // .service(
                    //     web::resource("/{user_id}")
                    //         .route(web::get().to(data_access_layer::user::User::get_user)), // .route(web::post().to(post_user))
                    //                                                                         // .route(web::delete().to(delete_user))
                    //                                                                         // .route(web::patch().to(update_user))
                    // ),
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
