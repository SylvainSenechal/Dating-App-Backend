use serde::{Deserialize, Serialize};

// USERS //////////////////////////////////////
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserInfosReq {
    pub uuid: String,
    pub name: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: Gender,
    pub looking_for: Gender,
    pub search_radius: u16,
    pub looking_for_age_min: u8,
    pub looking_for_age_max: u8,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Gender {
    #[serde(rename = "male")]
    Male,
    #[serde(rename = "female")]
    Female,
    #[serde(rename = "any")]
    Any,
}

use rusqlite::{types::ToSqlOutput, ToSql};

impl ToSql for Gender {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            Gender::Male => Ok("male".into()),
            Gender::Female => Ok("female".into()),
            Gender::Any => Ok("any".into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserRequest {
    pub name: String,
    pub password: String,
    pub email: String,
    pub age: u8,
    pub latitude: f32,
    pub longitude: f32,
    pub gender: Gender, // TODO add enum constraint
    pub looking_for: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwipeUserRequest {
    pub swiped_uuid: String,
    pub love: bool, // boolean for sqlite, 0 = dont love, 1 - love
}

// MESSAGES //////////////////////////////////////
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CreateMessageRequest {
    pub message: String,
    pub poster_uuid: String,
    pub love_uuid: String,
}

#[derive(Deserialize)]
pub struct GreenTickMessagesRequest {
    pub love_uuid: String,
    pub lover_ticked_uuid: String,
}
