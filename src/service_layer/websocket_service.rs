// use crate::service_layer::auth_service;
// use crate::{data_access_layer, AppState};
// use actix::prelude::Message as ActixMessage;
// use actix::{
//     fut, Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner,
//     Handler, Recipient, StreamHandler, WrapFuture,
// };
// use actix_web::{web, Error, HttpRequest, HttpResponse};
// use actix_web_actors::ws;
// use serde_json::json;
// use std::collections::{HashMap, HashSet};
// use uuid::Uuid;

// #[derive(ActixMessage)]
// #[rtype(result = "()")]
// pub struct Message(pub String);
// pub enum MessageType {
//     Chat,
//     GreenTick,
//     Info,
// }

// impl MessageType {
//     fn as_str(&self) -> &'static str {
//         match self {
//             MessageType::Chat => "chat",
//             MessageType::GreenTick => "green_tick",
//             MessageType::Info => "info",
//         }
//     }
// }

// #[derive(Debug)]
// pub struct Server {
//     pub sessions: HashMap<Uuid, Recipient<Message>>, // uuid user -> socket address
//     pub love_chat_rooms: HashMap<usize, HashSet<Recipient<Message>>>,
// }

// impl Actor for Server {
//     type Context = Context<Self>;
// }

// #[derive(ActixMessage, Debug)]
// #[rtype(result = "()")]
// pub struct Connect {
//     pub user_id: usize,
//     pub uuid: Uuid,
//     pub addr: Recipient<Message>,
// }

// #[derive(ActixMessage, Debug)]
// #[rtype(result = "()")]
// pub struct Disconnect {
//     pub user_id: usize,
//     pub uuid: Uuid,
//     pub love_rooms: HashSet<usize>,
// }

// impl Server {
//     fn send_chat_message(
//         &self,
//         love_room_id: usize,
//         poster_id: usize,
//         message: &str,
//         message_id: usize,
//         creation_datetime: String,
//     ) {
//         println!("sending messages");
//         println!("lovers r : {:?}", self.love_chat_rooms);
//         if let Some(lovers) = self.love_chat_rooms.get(&love_room_id) {
//             for lover in lovers {
//                 let message = json!({
//                     "message_type": MessageType::Chat.as_str().to_string(),
//                     "love_id": love_room_id,
//                     "message": message,
//                     "message_id": message_id,
//                     "poster_id": poster_id,
//                     "creation_datetime": creation_datetime
//                 });
//                 match lover.do_send(Message(message.to_string())) {
//                     Ok(_) => (),
//                     Err(e) => println!("Error while sending message to somebody : {}", e),
//                 }
//             }
//         }
//     }

//     fn send_message_was_green_ticked(&self, love_room_id: usize, message_id: usize) {
//         if let Some(lovers) = self.love_chat_rooms.get(&love_room_id) {
//             for lover in lovers {
//                 let message = json!({
//                     "message_type": MessageType::GreenTick.as_str().to_string(),
//                     "love_id": love_room_id,
//                     "message_id": message_id
//                 });
//                 match lover.do_send(Message(message.to_string())) {
//                     Ok(_) => (),
//                     Err(e) => {
//                         println!("Error while sending green tick message to somebody : {}", e)
//                     }
//                 }
//             }
//         }
//     }
// }

// #[derive(ActixMessage, Debug)]
// #[rtype(result = "()")]
// pub struct ChatMessage {
//     pub id_love_room: usize,
//     pub id_message: usize,
//     pub message: String,
//     pub poster_id: usize,
//     pub creation_datetime: String,
// }

// #[derive(ActixMessage, Debug)]
// #[rtype(result = "()")]
// pub struct GreenTickMessage {
//     pub id_love_room: usize,
//     pub id_message: usize,
// }

// #[derive(ActixMessage, Debug)]
// #[rtype(result = "()")]
// pub struct Join {
//     pub id_love_room: usize,
//     pub addr: Recipient<Message>, // addr is joining room id_love_room
// }
// impl Handler<Connect> for Server {
//     type Result = ();

//     // Register new websocket session + assign unique id
//     fn handle(&mut self, connection: Connect, _: &mut Context<Self>) {
//         // todo : Check that if user /authenticate multiple times, we are not Connecting multiple sessions for the same user
//         // At the moment, same user re connecting will override it's old session with a new one, but what if that person is connecting from both pc and smartphone ?
//         println!("someone joined : ");
//         println!("self        : {:?}", self);
//         println!("connection  : {:?}", connection);
//         // if self.sessions.contains_key(&connection.user_id) {
//         //     println!("User reconnecting from another client. Removing user from old chatrooms :");
//         //     self.sessions.remove(&connection.user_id);
//         // } else {
//         // }
//         self.sessions.insert(connection.uuid, connection.addr);
//         println!("SESSIONS 1 : {:?}", self.sessions);
//     }
// }

// impl Handler<Disconnect> for Server {
//     type Result = ();

//     fn handle(&mut self, disconnection: Disconnect, _: &mut Context<Self>) {
//         println!("SESSIONS 2 : {:?}", self.sessions);
//         println!("SESSIONS 3 : {:?}", disconnection.user_id);

//         let recipient_to_remove = self
//             .sessions
//             .get(&disconnection.uuid)
//             .expect("Tried getting a recipient that is not even in sessions");
//         for id_room in disconnection.love_rooms {
//             if let Some(val) = self.love_chat_rooms.get_mut(&id_room) {
//                 val.remove(recipient_to_remove);
//             }
//         }
//         self.sessions.remove(&disconnection.uuid);
//         println!("REMOVED {:?} ", self.love_chat_rooms);
//         println!("SESSIONS 4 : {:?}", self.sessions);
//     }
// }

// impl Handler<Join> for Server {
//     type Result = ();

//     // Register new websocket session + assign unique id
//     fn handle(&mut self, joining: Join, _: &mut Context<Self>) {
//         println!("joining room {}", joining.id_love_room);
//         self.love_chat_rooms
//             .entry(joining.id_love_room)
//             .or_insert_with(HashSet::new)
//             .insert(joining.addr);
//     }
// }
// impl Handler<ChatMessage> for Server {
//     type Result = ();

//     fn handle(&mut self, msg: ChatMessage, _: &mut Context<Self>) {
//         println!("yoyoyoyoyo");
//         self.send_chat_message(
//             msg.id_love_room,
//             msg.poster_id,
//             msg.message.as_str(),
//             msg.id_message,
//             msg.creation_datetime,
//         );
//     }
// }

// impl Handler<GreenTickMessage> for Server {
//     type Result = ();

//     fn handle(&mut self, msg: GreenTickMessage, _: &mut Context<Self>) {
//         println!("yoyoyoyoyo");
//         self.send_message_was_green_ticked(msg.id_love_room, msg.id_message);
//     }
// }

// /////////////////////////////

// #[derive(Debug)]
// pub struct MyWs {
//     db_connection: web::Data<AppState>,
//     user_id: Option<usize>, // None => Websocket not authentified
//     uuid: Option<Uuid>,
//     ids_joined_rooms: HashSet<usize>, // All the love room of the user <=> All the person you've matched with and can talk with
//     addr: Addr<Server>,
// }

// impl Actor for MyWs {
//     type Context = ws::WebsocketContext<Self>;

//     fn started(&mut self, _: &mut Self::Context) {
//         println!("Websocket connection started");
//     }

//     fn stopped(&mut self, _: &mut Self::Context) {
//         println!("Websocket connection stopped");
//         self.addr.do_send(Disconnect {
//             user_id: self.user_id.unwrap(),
//             uuid: self.uuid.unwrap(),
//             love_rooms: self.ids_joined_rooms.clone(),
//         })
//     }
// }

// impl Handler<Message> for MyWs {
//     // The websocket receives messages from the server, and sends them back to the user
//     type Result = ();

//     fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
//         println!("a texxxxxxt {} ", msg.0);
//         ctx.text(msg.0)
//     }
// }

// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
//     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
//         println!("myws message {:?}", msg);
//         println!("myws seflfff {:?}", self);
//         println!("myws address {:?}", ctx.address());
//         let msg = match msg {
//             Err(_) => {
//                 ctx.stop();
//                 return;
//             }
//             Ok(msg) => msg,
//         };

//         // TODO : this stuff might need some refactoring -_-
//         match msg {
//             ws::Message::Text(text) => {
//                 println!("Socket received this message : {}", text);
//                 let m = text.trim();
//                 if m.starts_with('/') {
//                     let v: Vec<&str> = m.splitn(2, ' ').collect();
//                     println!("message : {}", m);
//                     println!("v split : {:?}", v);

//                     match v[0] {
//                         "/authenticate" => {
//                             if v.len() == 2 {
//                                 println!("Authenticating user : {}", v[1]);
//                                 let token = v[1];
//                                 match auth_service::validate_token(token) {
//                                     Some(authorized) => {
//                                         println!("Authentication successfull");
//                                         self.user_id = Some(authorized.id);
//                                         self.uuid = Some(Uuid::new_v4());

//                                         let addr = ctx.address();
//                                         // Register the websocket session in the server :
//                                         self.addr
//                                             .send(Connect {
//                                                 user_id: authorized.id,
//                                                 uuid: self.uuid.unwrap(),
//                                                 addr: addr.recipient(),
//                                             })
//                                             .into_actor(self)
//                                             .then(|res, _, ctx| {
//                                                 match res {
//                                                     Ok(_) => println!("Added user to websocket server successfully"),
//                                                     _ => ctx.stop(),
//                                                 }
//                                                 fut::ready(())
//                                             })
//                                             .wait(ctx);

//                                         match data_access_layer::lover_dal::get_lovers(
//                                                 &self.db_connection,
//                                                 authorized.id,
//                                             ) {
//                                                 Ok(lovers) => {
//                                                     for lover in lovers {
//                                                         self.addr.do_send(Join {
//                                                             id_love_room: lover.love_id,
//                                                             addr: ctx.address().recipient(),
//                                                         });
//                                                         self.ids_joined_rooms.insert(lover.love_id);
//                                                     }

//                                                     // self.ids_joined_rooms.insert(room_id);
//                                                 },
//                                                     Err(err) => println!("Error when user_id : {} tried finding lovers, error is : {}", authorized.id, err)
//                                             }

//                                         let message = json!({
//                                             "message_type": MessageType::Info.as_str().to_string(),
//                                             "message": "Authentication successfull"
//                                         });
//                                         ctx.text(message.to_string())
//                                     }
//                                     None => {
//                                         println!("Wrong command format");
//                                         let message = json!({
//                                             "message_type": MessageType::Info.as_str().to_string(),
//                                             "message": "Wrong command format"
//                                         });
//                                         ctx.text(message.to_string())
//                                     }
//                                 }
//                             } else {
//                                 println!("Could not authenticate on websocket");
//                                 let message = json!({
//                                     "message_type": MessageType::Info.as_str().to_string(),
//                                     "message": "Could not authenticate on websocket"
//                                 });
//                                 ctx.text(message.to_string())
//                             }
//                         }
//                         other_commands => {
//                             match self.user_id {
//                                 Some(user_id) => {
//                                     // The other commands can only be performed when user is authenticated
//                                     match other_commands {
//                                         command => println!("unknown /command : {}", command),
//                                     }
//                                 }
//                                 None => {
//                                     println!("Performing a command without being authenticated");
//                                     let message = json!({
//                                         "message_type": MessageType::Info.as_str().to_string(),
//                                         "message": "Performing a command without being authenticated"
//                                     });
//                                     ctx.text(message.to_string())
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//             _ => println!("Message type unexpected {:?}", msg),
//         }
//     }
// }

// pub async fn index_websocket(
//     req: HttpRequest,
//     db: web::Data<AppState>,
//     stream: web::Payload,
//     server: web::Data<Addr<Server>>,
// ) -> Result<HttpResponse, Error> {
//     let resp = ws::start(
//         MyWs {
//             db_connection: db,
//             user_id: None,
//             uuid: None, // Using uuid, so one user can connect from different clients, user's 2 clients will be in the same room and will both receive event
//             ids_joined_rooms: HashSet::new(),
//             addr: server.get_ref().clone(),
//         },
//         &req,
//         stream,
//     );

//     resp
// }
