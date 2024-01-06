use juniper::Context;
use rocket_db_pools::Database;

#[derive(Database)]
#[database("sqlx")]
pub struct Db(sqlx::SqlitePool);

impl Context for Db {}

impl Db {
    pub fn new() -> Self {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool =
            sqlx::SqlitePool::connect_lazy(db_url.as_str()).expect("Failed to create DB pool");
        Self(pool)
    }
}

pub type DbResult<T, E = rocket::response::Debug<sqlx::Error>> = Result<T, E>;
