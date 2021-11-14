use actix_web::{
    dev::Service,
    http::{header, Method, StatusCode},
    Error as AWError, HttpRequest,
};
use rusqlite::{params, Connection, Error as sqliteError, Result as resultSQLite, Statement};
use serde::{Deserialize, Serialize};

use actix_web::{
    get, guard, middleware, web, App, HttpResponse, HttpServer, Responder, Result as actixResult,
};
use std::fmt;
// use libsqlite3_sys::Error as errorLibSqlite3;

// TODO : see and_then()

const DATABASE_NAME: &str = "love.db";
// const QUERY_GET_PERSONS: &str = "get_persons";
// const QUERY_CREATE_PERSONS: &str = "create_persons";

struct AppState {
    connection: Connection,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct User {
    #[serde(default)]
    age: u8,
    #[serde(default)]
    name: String,
    #[serde(default)]
    email: String, // use option instead, see what returning json option does (maybe serde remove option none ?)
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
                name TEXT NOT NULL UNIQUE,
                age INTEGER
            )",
                [],
            )
            .expect("Could not create table persons");
    }
}

// serde
// serialize puis deserialize avec donnees manquantes et defaut

impl User {
    fn create_person(connection: &Connection, person: User) -> resultSQLite<()> {
        let mut statement = connection
            .prepare_cached("INSERT INTO persons (age, name) VALUES (?, ?)")
            .expect("Could not generate prepared statements");
        statement.execute(params![person.age, person.name])?;

        Ok(())
    }

    fn get_person(connection: &Connection, name: &str) -> resultSQLite<User> {
        let mut statement = connection
            .prepare("SELECT * FROM persons WHERE name = ?")
            .expect("Could not generate prepared statements");
        statement.query_row(params![name], |row| {
            let json = r#"
                {
                    "name": "ououi"
                }
            "#;

            let pe: User = serde_json::from_str(json).unwrap();
            println!("serdeeee: {:?}", pe);

            Ok(User {
                age: row.get(1)?,
                name: row.get(2)?,
                ..Default::default()
            })
        })
    }

    fn get_persons(connection: &Connection) -> resultSQLite<Vec<User>> {
        let mut statement = connection
            .prepare("SELECT * FROM persons")
            .expect("Could not generate prepared statements");
        let result_rows = statement.query_map([], |row| {
            Ok(User {
                age: row.get(1)?,
                name: row.get(2)?,
                ..Default::default()
            })
        })?;
        let mut persons = Vec::new();
        for person in result_rows {
            persons.push(person.expect("Could not unwrap on result_rows get persons"));
        }
        Ok(persons)
    }
}

// async fn p404() -> Result<fs::NamedFile> {
//     Ok(fs::NamedFile::open("static/404.html")?.set_status_code(StatusCode::NOT_FOUND))
// }

#[derive(Debug)]
enum CustomError {
    NotFound,
    Unknown,
    SqliteFailure(libsqlite3_sys::Error),
    SqliteFailureNoText,
}
impl CustomError {
    pub fn name(&self) -> String {
        match self {
            Self::NotFound => "Ressource not found".to_string(),
            Self::SqliteFailure(sqliteFailureDetail) => sqliteFailureDetail.to_string(),
            Self::SqliteFailureNoText => {
                "Something fucked up in the database, it's not your fault dud".to_string()
            }
            Self::Unknown => "Unknow error".to_string(),
        }
    }
}
impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Serialize)]

struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
impl actix_web::ResponseError for CustomError {
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::SqliteFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqliteFailureNoText => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
            error: self.name(),
        };
        HttpResponse::build(status_code).json(error_response)
    }
}

fn map_sqlite_error(e: rusqlite::Error) -> CustomError {
    // Todo : Add this to the logger
    println!("map sqlite error found : {:?}", e);

    match e {
        rusqlite::Error::QueryReturnedNoRows => CustomError::NotFound,
        rusqlite::Error::SqliteFailure(sqliteFailureDetail, Some(_)) => CustomError::SqliteFailure(sqliteFailureDetail),
        rusqlite::Error::SqliteFailure(sqliteFailureDetail, None) => CustomError::SqliteFailure(sqliteFailureDetail),
        rusqlite::Error::InvalidColumnIndex(_) => CustomError::SqliteFailureNoText,
        rusqlite::Error::InvalidColumnType(_, _, _) => CustomError::SqliteFailureNoText,

        _ => CustomError::NotFound,
    }
}

async fn get_user(
    db: web::Data<AppState>,
    web::Path(id): web::Path<u32>,
    user: web::Json<User>,
    req: HttpRequest,
) -> actixResult<HttpResponse, CustomError> {
    println!("{:?}", db.connection);
    println!("{:?}", user);
    println!("{:?}", req);
    println!("{:?}", id);

    let mut statement = db
        .connection
        .prepare("SELECT * FROM persons WHERE name = ?")
        .map_err(map_sqlite_error)?;

    let res = statement
        .query_row(params![user.name], |row| {
            Ok(User {
                age: row.get(1)?,
                name: row.get(2)?,
                ..Default::default()
            })
        })
        .map_err(map_sqlite_error)?;

    Ok(HttpResponse::Ok().json(res))
}

async fn post_user(
    db: web::Data<AppState>,
    user: web::Json<User>,
) -> actixResult<HttpResponse, CustomError> {
    let mut statement = db
        .connection
        .prepare("INSERT INTO persons (name) VALUES (?)")
        .map_err(map_sqlite_error)?;
    let nb_inserted = statement
        .execute(params![user.name])
        .map_err(map_sqlite_error)?;
    println!("inserted  : {:?}", nb_inserted);

    Ok(HttpResponse::Ok().body("")) //.json(res))
}

async fn get_users(db: web::Data<AppState>) -> actixResult<HttpResponse, CustomError> {
    let mut statement = db
        .connection
        .prepare("SELECT * FROM persons")
        .map_err(map_sqlite_error)?;
    let result_rows = statement
        .query_map([], |row| {
            Ok(User {
                age: row.get(1)?,
                name: row.get(2)?,
                ..Default::default()
            })
        })
        .map_err(map_sqlite_error)?;

    let mut persons = Vec::new();
    for person in result_rows {
        persons.push(person.map_err(map_sqlite_error)?);
    }

    Ok(HttpResponse::Ok().json(persons))
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
                            .route(web::get().to(get_users))
                            .route(web::post().to(post_user)),
                    )
                    .service(
                        web::resource("/{user_id}").route(web::get().to(get_user)), // .route(web::post().to(post_user))
                                                                                    // .route(web::delete().to(delete_user))
                                                                                    // .route(web::patch().to(update_user))
                    ),
            )
            .default_service(
                web::resource("")
                    // .route(web::get().to(p404))
                    .route(
                        web::route()
                            .guard(guard::Not(guard::Get()))
                            .to(HttpResponse::MethodNotAllowed),
                    ),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await

    // let p = Person{age: 25, name: "sylvain".to_string(), ..Default::default()};
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
