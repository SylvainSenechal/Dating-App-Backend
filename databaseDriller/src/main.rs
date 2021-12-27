use futures::StreamExt;
use rand::Rng;
use reqwest;
use serde::{Deserialize, Serialize};
use tokio_stream; 

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
    println!("Hello, world!");
    let client = reqwest::Client::new();
    let person = UserLoginRequest {
        pseudo: "aa".to_string(),
        password: "aa".to_string(),
    };


    let fetches = tokio_stream::iter((0..50).map(|id| {
        println!("{}", id);

        async {
            // println!("mew request started");
            let a = rand::thread_rng().gen_range(0..10);
            if a >= 50 {
                let resp = client.get("http://localhost:8080/users/unUgser").send().await;//.unwrap();
                match resp {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{}", e.to_string());
                        panic!()
                    }
                }
                let resp = resp.unwrap().bytes().await;
                // println!("{:?}", resp.unwrap());

                return PersonResponse {
                    jwt_token: "XXXXXXX".to_string(),
                    message: "AAAAAAAAA".to_string(),
                };
            } else {
                let response = client
                    .post("http://localhost:8080/auth")
                    .json(&person)
                    .send()
                    .await
                    .unwrap();

                let res = response.json::<PersonResponse>().await.unwrap();
                println!("{:?}", res);
                // let res = PersonResponse{jwt_token: "oui".to_string(), message: "ok".to_string()};
                return res;
            }
        }
    }))
    .buffer_unordered(5)
    .collect::<Vec<_>>();

    let a = fetches.await;

    // print!("{:?}", a);
    println!("end");

    Ok(())
}