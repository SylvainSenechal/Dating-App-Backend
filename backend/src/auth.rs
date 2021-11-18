// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
// ==> "Use Argon2id with a minimum configuration of 15 MiB of memory, an iteration count of 2, and 1 degree of parallelism."
// use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
// use argon2::{
//     password_hash::{
//         rand_core::OsRng,
//         PasswordHash, PasswordHasher, PasswordVerifier, SaltString
//     },
//     Argon2
// };


const KEY_JWT: &str = "badObviousTestKey";
// const KEY_JWT_REFRESH = "ohohoho"
// const TOKEN_LIFESPAN = "30sec"
// const TOKEN_REFRESH_LIFESPAN = "3600sec"

// pub fn coucou() {

// }