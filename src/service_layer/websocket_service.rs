use actix::prelude::Message as ActixMessage;
use actix::{
    fut, Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner,
    Handler, Recipient, StreamHandler, WrapFuture,
};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::ops::Add;
use serde_json::json;

use crate::service_layer::auth_service;

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Message(pub String);
pub enum MessageType {
    Chat,
    Info,
}

impl MessageType { // TODO use JSON serde retuned message everywhere
    fn as_str(&self) -> &'static str {
        match self {
            MessageType::Chat => "chat",
            MessageType::Info => "info",
        }
    }
}

#[derive(Debug)]
pub struct Server {
    pub sessions: HashMap<u32, Recipient<Message>>, // id user -> socket address
    pub love_chat_rooms: HashMap<u32, HashSet<Recipient<Message>>>,
}

impl Actor for Server {
    type Context = Context<Self>;
}

#[derive(ActixMessage, Debug)]
#[rtype(usize)]
pub struct Connect {
    pub user_id: u32,
    pub addr: Recipient<Message>,
}

#[derive(ActixMessage, Debug)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub user_id: u32,
    pub love_rooms: HashSet<u32>,
}

impl Server {
    fn send_chat_message(&self, love_room: u32, poster_id: u32, message: &str, message_id: i64) {
        println!("sending messages");
        println!("lovers r : {:?}", self.love_chat_rooms);
        if let Some(lovers) = self.love_chat_rooms.get(&love_room) {
            for lover in lovers {
                // match lover.do_send(Message( // TODO : check if string literal or some better concat technique..
                //     MessageType::Chat
                //         .as_str()
                //         .to_string()
                //         .add(&message_id.to_string())
                //         .add(" ")
                //         .add(message),
                // )) {
                //     Ok(_) => (),
                //     Err(e) => println!("Error while sendin message to somebody : {}", e),
                // }
                let m = json!({
                    "message_type": MessageType::Chat.as_str().to_string(),
                    "love_id": love_room,
                    "message": message,
                    "message_id": message_id,
                    "poster_id": poster_id
                });
                match lover.do_send(Message(m.to_string())) {
                    Ok(_) => (),
                    Err(e) => println!("Error while sendin message to somebody : {}", e),
                }
            }
        }
    }
}

#[derive(ActixMessage, Debug)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub id_love_room: u32,
    pub id_message: i64,
    pub message: String,
    pub poster_id: u32,
}
#[derive(ActixMessage, Debug)]
#[rtype(result = "()")]
pub struct Join {
    pub id_love_room: u32,
    pub addr: Recipient<Message>, // addr is joining room id_love_room
}
impl Handler<Connect> for Server {
    type Result = usize;

    // Register new websocket session + assign unique id
    fn handle(&mut self, connection: Connect, _: &mut Context<Self>) -> Self::Result {
        println!("someone joined : ");
        println!("self        : {:?}", self);
        println!("connection  : {:?}", connection);
        let id: usize = rand::thread_rng().gen::<usize>();
        self.sessions.insert(connection.user_id, connection.addr);
        id
    }
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, disconnection: Disconnect, _: &mut Context<Self>) {
        let recipient_to_remove = self
            .sessions
            .get(&disconnection.user_id)
            .expect("Tried removing a recipient that is not even in sessions");
        for id_room in disconnection.love_rooms {
            if let Some(val) = self.love_chat_rooms.get_mut(&id_room) {
                val.remove(&recipient_to_remove);
            }
        }
        self.sessions.remove(&disconnection.user_id);
    }
}

impl Handler<Join> for Server {
    type Result = ();

    // Register new websocket session + assign unique id
    fn handle(&mut self, joining: Join, _: &mut Context<Self>) {
        self.love_chat_rooms
            .entry(joining.id_love_room)
            .or_insert_with(HashSet::new)
            .insert(joining.addr);
    }
}
impl Handler<ChatMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, _: &mut Context<Self>) {
        self.send_chat_message(msg.id_love_room, msg.poster_id, msg.message.as_str(), msg.id_message);
    }
}

/////////////////////////////

#[derive(Debug)]
pub struct MyWs {
    user_id: Option<u32>,           // None => Websocket not authentified
    id_love_room: Option<u32>,      // None => Haven't joined any love room
    ids_joined_rooms: HashSet<u32>, // All the love room of the user <=> All the person you've matched with and can talk with
    addr: Addr<Server>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Websocket connection started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Websocket connection stopped");
        self.addr.do_send(Disconnect {
            user_id: self.user_id.unwrap(),
            love_rooms: self.ids_joined_rooms.clone(),
        })
    }
}

impl Handler<Message> for MyWs {
    // The websocket receives messages from the server, and sends them back to the user
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        println!("a texxxxxxt");
        ctx.text(msg.0)
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("myws message {:?}", msg);
        println!("myws seflfff {:?}", self);
        println!("myws address {:?}", ctx.address());
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        // TODO : this stuff might need some refactoring -_-
        match msg {
            ws::Message::Text(text) => {
                // TODO : add this message to the logs
                let m = text.trim();
                if m.starts_with("/") {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    println!("message : {}", m);
                    println!("v split : {:?}", v);

                    match v[0] {
                        "/authenticate" => {
                            // TODO ! : This might be a dirty auth..
                            if v.len() == 2 {
                                println!("Authenticating user : {}", v[1]);
                                let token = v[1];
                                match auth_service::validate_token(token) {
                                    Some(authorized) => {
                                        println!("Authentication successfull");
                                        self.user_id = Some(authorized.id);
                                        // todo : Check that if user /authenticate multiple times, we are not Connection multiple sessions -_-..
                                        let addr = ctx.address();
                                        // Register the websocket session in the server :
                                        self.addr
                                            .send(Connect {
                                                user_id: authorized.id,
                                                addr: addr.recipient(),
                                            })
                                            .into_actor(self)
                                            .then(|res, _, ctx| {
                                                match res {
                                                    Ok(_) => (),
                                                    _ => ctx.stop(), // TODO : Do I really want to do that ?
                                                }
                                                fut::ready(())
                                            })
                                            .wait(ctx);
                                        ctx.text("Authentication successfull")
                                    }
                                    None => {
                                        println!("Can't identify");
                                        ctx.text("Can't authenticate")
                                    }
                                }
                            } else {
                                println!("error TODO handle");
                            }
                        }
                        other_commands => {
                            match self.user_id {
                                Some(_) => {
                                    // The other commands can only be performed when user is authenticated
                                    match other_commands {
                                        "/join" => {
                                            if v.len() == 2 {
                                                // TODO : ensure that user is allowed to join room (loved by the other..)
                                                println!("joining room : {}", v[1]);
                                                self.addr.do_send(Join {
                                                    id_love_room: v[1].parse().unwrap(),
                                                    addr: ctx.address().recipient(),
                                                });
                                                self.id_love_room = Some(v[1].parse().unwrap());
                                                self.ids_joined_rooms.insert(v[1].parse().unwrap());
                                            } else {
                                                println!("error TODO handle");
                                            }
                                        }
                                        // No need this stuff, message sent from message_service.rs
                                        // "/message" => {
                                        //     if v.len() == 2 {
                                        //         self.addr.do_send(ChatMessage {
                                        //             id_love_room: self
                                        //                 .id_love_room
                                        //                 .expect("User sends message but havent joined a room"),
                                        //             message: v[1].parse().unwrap(),
                                        //         });
                                        //         println!("joining room : {}", v[1]);
                                        //     } else {
                                        //         println!("error TODO handle");
                                        //     }
                                        // }
                                        _ => println!("unknown /command"), // todo handle
                                    }
                                }
                                None => {
                                    println!("Performing a command without being authenticated")
                                }
                            }
                        }
                    }
                }
            }
            _ => println!("Message type unexpected {:?}", msg), // todo handle
        }
    }
}

pub async fn index_websocket(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<Server>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        MyWs {
            user_id: None,
            id_love_room: None,
            ids_joined_rooms: HashSet::new(), // Todo : fill this with all the relevant love rooms
            addr: server.get_ref().clone(),
        },
        &req,
        stream,
    );

    resp
}
