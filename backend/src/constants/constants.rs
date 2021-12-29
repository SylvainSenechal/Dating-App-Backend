// TODO : voir const vs static 

// https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
// ==> "Use Argon2id with a minimum configuration of 15 MiB of memory, an iteration count of 2, and 1 degree of parallelism."
pub const M_COST: u32 = 15_000;// m_cost is the memory size, expressed in kilobytes
pub const T_COST: u32 = 1; // t_cost is the number of iterations;
pub const P_COST: u32 = 1; //p_cost is the degree of parallelism.
pub const OUTPUT_LEN: usize = 32; // determines the length of the returned hash in bytes

pub const DATABASE_NAME: &str = "love.db";

pub const BEARER: &str = "Bearer ";
pub const KEY_JWT: &[u8] = b"badObviousTestKey";
pub const KEY_JWT_REFRESH: &[u8] = b"ohohoho";
pub const TOKEN_LIFESPAN: usize = 10; // seconds
pub const TOKEN_REFRESH_LIFESPAN: usize = 3600; // seconds
pub const DEFAULT_HASH: &str = "$argon2id$v=19$m=15000,t=2,p=1$SZZVht0nCXacXAJU1dYJ8w$QwpNt6gUQ2K+dHQVDTf5H1mkkA0yTkXXKwZ6vHkKClQ";
