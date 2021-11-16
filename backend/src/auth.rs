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
use argon2::{
    Algorithm, Argon2, Error, Params, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use hex_literal::hex;


const KEY_JWT: &str = "badObviousTestKey";

// const KEY_JWT_REFRESH = "ohohoho"
// const TOKEN_LIFESPAN = "30sec"
// const TOKEN_REFRESH_LIFESPAN = "3600sec"

pub fn coucou() {
    println!("mCOUCOUCOUC");

    // let password = b"hunter42"; // Bad password; don't actually use!
    // let salt = SaltString::generate(&mut OsRng);
    
    // // Argon2 with default params (Argon2id v19)
    // let argon2 = Argon2::default();
    
    // // Hash password to PHC string ($argon2id$v=19$...)
    // let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();
    
    // // Verify password against PHC string
    // let parsed_hash = PasswordHash::new(&password_hash).unwrap();
    // assert!(argon2.verify_password(password, &parsed_hash).is_ok());



    let algorithm = Algorithm::Argon2id;
    let version = Version::V0x13;
    let params = example_params();
    let password = [0x01; 32];
    let salt = [0x02; 16];
    let secret = [0x03; 8];
    let expected_tag = hex!(
        "
        0d 64 0d f5 8d 78 76 6c 08 c0 37 a3 4a 8b 53 c9
        d0 1e f0 45 2d 75 b6 5e b5 25 20 e9 6b 01 e6 59
        "
    );

    let ctx = Argon2::new_with_secret(&secret, algorithm, version, params).unwrap();
    let mut out = [0u8; 32];
    ctx.hash_password_into(&password, &salt, &mut out).unwrap();

    assert_eq!(out, expected_tag);
    println!("eeeee : {:?}", out);
    println!("eeeee : {:?}", expected_tag);
    println!("eeeee : {:?}", password);

}

fn example_params() -> Params {
    let mut builder = ParamsBuilder::new();
    builder.m_cost(32).unwrap();
    builder.t_cost(3).unwrap();
    builder.p_cost(4).unwrap();
    builder.data(&[0x04; 12]).unwrap();
    builder.params().unwrap()
}