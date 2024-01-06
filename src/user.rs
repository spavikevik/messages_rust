use juniper::futures::TryStreamExt;
use juniper::graphql_object;
use rocket::http::hyper::body::HttpBody;
use rocket_db_pools::Pool;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, Executor, FromRow, Row, Sqlite};
use time::OffsetDateTime;
use uuid::{Bytes, Uuid};

use crate::db::*;
use crate::message::Message;

pub struct User {
    id: Option<Uuid>,
    display_name: String,
    username: String,
    password_hash: String,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
}

impl FromRow<'_, SqliteRow> for User {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, Error> {
        Ok(Self {
            id: Some(Self::convert_to_uuid(row.try_get("id")?)),
            display_name: row.try_get("display_name")?,
            username: row.try_get("username")?,
            password_hash: row.try_get("password_hash")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl User {
    pub fn new(display_name: &str, username: &str, password_hash: &str) -> Self {
        Self {
            id: None,
            display_name: display_name.to_string(),
            username: username.to_string(),
            password_hash: password_hash.to_string(),
            created_at: None,
            updated_at: None,
        }
    }

    pub async fn insert(&self, mut conn: PoolConnection<Sqlite>) -> DbResult<Self> {
        let uuid = Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes() as &[u8];

        let query = sqlx::query!(
            "INSERT INTO users (id, display_name, username, password_hash) VALUES (?, ?, ?, ?) RETURNING *",
            uuid_bytes, self.display_name, self.username, self.password_hash
        );

        let result: SqliteRow = conn.fetch_one(query).await?;
        let user = User::from_row(&result)?;

        Ok(user)
    }

    pub async fn get(id: &Uuid, mut conn: PoolConnection<Sqlite>) -> DbResult<Self> {
        let id_bytes = id.as_bytes() as &[u8];
        let row = conn
            .fetch_one(sqlx::query!("SELECT * FROM users WHERE id = ?", id_bytes))
            .await?;

        let user = User::from_row(&row)?;

        Ok(user)
    }

    fn convert_to_uuid(bytes: Vec<u8>) -> Uuid {
        Uuid::from_bytes(Bytes::try_from(bytes.as_slice()).unwrap())
    }
}

#[graphql_object(context = Db)]
impl User {
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub async fn messages(&self, ctx: &Db) -> Option<Vec<Message>> {
        match self.id {
            None => None,
            Some(id) => {
                let connection = ctx.get().await.ok()?;
                Message::get_for_user(&id, connection).await.ok()?
            }
        }
    }

    pub fn created_at(&self) -> Option<OffsetDateTime> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<OffsetDateTime> {
        self.updated_at
    }
}
