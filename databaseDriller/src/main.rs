use futures::StreamExt;
use futures::stream;
use rand::Rng;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio_stream; 
use std::time::SystemTime;
use std::time::Duration;
use tokio::task;

#[derive(Debug, Serialize, Deserialize)]
struct UserLoginRequest {
    pseudo: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersonResponse {
    jwt_token: String,
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    console_subscriber::init();
    let client = reqwest::Client::new();
    let person = UserLoginRequest {
        pseudo: "aa".to_string(),
        password: "aa".to_string(),
    };
    let now = SystemTime::now();


    // let fetches = tokio_stream::iter((0..500_000).map(|id| {
    //     println!("{}", id);

    //     async {
    //         // println!("mew request started");
    //         let a = rand::thread_rng().gen_range(0..10);
    //         if a >= 0 {
    //             // let resp = client.get("http://localhost:8080/users/aa").send().await;//.unwrap();
    //             let resp = client.get("http://localhost:8080/hey").send().await;//.unwrap();
    //             match resp {
    //                 Ok(_) => (),
    //                 Err(e) => {
    //                     println!("{}", e.to_string());
    //                     panic!()
    //                 }
    //             }
    //             let resp = resp.unwrap().bytes().await;
    //             // println!("{:?}", resp.unwrap());
    //             // return PersonResponse {
    //             //     jwt_token: "XXXXXXX".to_string(),
    //             //     message: "AAAAAAAAA".to_string(),
    //             // };
    //         } else {
    //             let response = client
    //                 .post("http://localhost:8080/auth")
    //                 .json(&person)
    //                 .send()
    //                 .await
    //                 .unwrap();

    //             let res = response.json::<PersonResponse>().await.unwrap();
    //             println!("{:?}", res);
    //             // let res = PersonResponse{jwt_token: "oui".to_string(), message: "ok".to_string()};
    //             // return res;
    //         }
    //     }
    // }))
    // .buffer_unordered(20)
    // .collect::<()>();



    let fetches = tokio_stream::iter((0..1000_000).map(|id| {
        println!("{}", id);

        async {
            // let resp = client.get("http://localhost:8080/users/aa").send().await;//.unwrap();
            let resp = client.get("http://localhost:8080/").send().await;//.unwrap();
            match resp {
                Ok(_) => (),
                Err(e) => {
                    println!("{}", e.to_string());
                    panic!()
                }
            }
            let resp = resp.unwrap().bytes().await;
            // println!("{:?}", resp.unwrap());
            // return PersonResponse {
            //     jwt_token: "XXXXXXX".to_string(),
            //     message: "AAAAAAAAA".to_string(),
            // };
        }
    }))
    .buffer_unordered(50)
    .collect::<()>();

    let a = fetches.await;
    // print!("{:?}", a);

    println!("{:?}", now.elapsed());
    Ok(())
}