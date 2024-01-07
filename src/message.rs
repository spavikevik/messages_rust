use crate::db::{Db, DbResult};
use juniper::graphql_object;
use rocket_db_pools::Pool;
use sqlx::pool::PoolConnection;
use sqlx::sqlite::SqliteRow;
use sqlx::{Error, Executor, FromRow, Row, Sqlite};
use std::mem;
use time::OffsetDateTime;
use uuid::{Bytes, Uuid};

pub struct Message {
    id: Option<Uuid>,
    user_id: Uuid,
    content: String,
    parent_message_id: Option<Uuid>,
    created_at: Option<OffsetDateTime>,
    updated_at: Option<OffsetDateTime>,
}

impl FromRow<'_, SqliteRow> for Message {
    fn from_row(row: &'_ SqliteRow) -> Result<Self, Error> {
        let parent_message_id: Option<Vec<u8>> = row.try_get("parent_message_id")?;

        Ok(Self {
            id: Some(Message::convert_to_uuid(row.try_get("id")?)),
            user_id: Message::convert_to_uuid(row.try_get("user_id")?),
            content: row.try_get("content")?,
            parent_message_id: parent_message_id.map(Message::convert_to_uuid),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl Message {
    pub fn new(user_id: Uuid, content: &str, parent_message_id: Option<Uuid>) -> Message {
        Self {
            id: None,
            user_id,
            content: content.to_string(),
            parent_message_id,
            created_at: None,
            updated_at: None,
        }
    }

    pub async fn insert(&self, mut conn: PoolConnection<Sqlite>) -> DbResult<Message> {
        let uuid = Uuid::new_v4();
        let uuid_bytes = Message::convert_uuid_to_bytes(&uuid);
        let user_id_bytes = Message::convert_uuid_to_bytes(&self.user_id);
        let parent_message_id_bytes = self
            .parent_message_id
            .as_ref()
            .map(|uuid| Message::convert_uuid_to_bytes(uuid));

        let query = sqlx::query!(
            "INSERT INTO messages (id, user_id, content, parent_message_id) VALUES (?, ?, ?, ?) RETURNING *",
            uuid_bytes, user_id_bytes, self.content, parent_message_id_bytes
        );

        let result: SqliteRow = conn.fetch_one(query).await?;
        let message = Message::from_row(&result)?;

        Ok(message)
    }

    pub async fn update(
        id: &Uuid,
        new_content: String,
        mut conn: PoolConnection<Sqlite>,
    ) -> DbResult<Message> {
        let id_bytes = Message::convert_uuid_to_bytes(id);
        let query = sqlx::query!(
            "UPDATE messages SET content = ? WHERE id = ? RETURNING *",
            id_bytes,
            new_content
        );

        let result: SqliteRow = conn.fetch_one(query).await?;
        let message = Message::from_row(&result)?;

        Ok(message)
    }

    pub async fn get(id: &Uuid, mut conn: PoolConnection<Sqlite>) -> DbResult<Message> {
        let id_bytes = Message::convert_uuid_to_bytes(id);
        let query = sqlx::query!("SELECT * FROM messages WHERE id = ?", id_bytes);

        let result: SqliteRow = conn.fetch_one(query).await?;
        let message = Message::from_row(&result)?;

        Ok(message)
    }

    pub async fn get_by_time_range(
        user_id: Option<&Uuid>,
        time_range: (OffsetDateTime, OffsetDateTime),
        mut conn: PoolConnection<Sqlite>,
    ) -> DbResult<Option<Vec<Message>>> {
        let user_id_bytes = user_id.map(Message::convert_uuid_to_bytes);
        let (after, before) = time_range;

        let result_future = match user_id_bytes {
            None => conn.fetch_all(sqlx::query!(
                "SELECT * FROM messages WHERE DATETIME(created_at) >= DATETIME(?) AND DATETIME(created_at) <= DATETIME(?)",
                after,
                before
            )),
            Some(_) => conn.fetch_all(sqlx::query!(
                "SELECT * FROM messages WHERE DATETIME(created_at) >= DATETIME(?) AND DATETIME(created_at) <= DATETIME(?) AND user_id = DATETIME(?)",
                after,
                before,
                user_id_bytes
            )),
        };

        let result: Vec<SqliteRow> = result_future.await?;

        let messages = &mut vec![];

        result.iter().for_each(|row| {
            let message = Message::from_row(row);
            match message {
                Ok(msg) => messages.push(msg),
                _ => (),
            }
        });

        if messages.is_empty() {
            Ok(None)
        } else {
            Ok(Some(mem::take(messages)))
        }
    }

    pub async fn get_for_user(
        user_id: &Uuid,
        mut conn: PoolConnection<Sqlite>,
    ) -> DbResult<Option<Vec<Message>>> {
        let id_bytes = Message::convert_uuid_to_bytes(user_id);
        let query = sqlx::query!(
            "SELECT * FROM messages WHERE parent_message_id = ?",
            id_bytes
        );

        let result: Vec<SqliteRow> = conn.fetch_all(query).await?;
        let messages = &mut vec![];

        result.iter().for_each(|row| {
            let message = Message::from_row(row);
            match message {
                Ok(msg) => messages.push(msg),
                _ => (),
            }
        });

        if messages.is_empty() {
            Ok(None)
        } else {
            Ok(Some(mem::take(messages)))
        }
    }

    pub async fn delete(id: Uuid, mut conn: PoolConnection<Sqlite>) -> DbResult<Message> {
        let id_bytes = Message::convert_uuid_to_bytes(&id);
        let query = sqlx::query!("DELETE FROM messages WHERE id = ? RETURNING *", id_bytes);

        let result: SqliteRow = conn.fetch_one(query).await?;
        let message = Message::from_row(&result)?;

        Ok(message)
    }

    async fn get_replies(
        &self,
        mut conn: PoolConnection<Sqlite>,
    ) -> DbResult<Option<Vec<Message>>> {
        match self.id {
            None => Ok(None),
            Some(id) => {
                let id_bytes = Message::convert_uuid_to_bytes(&id);
                let query = sqlx::query!(
                    "SELECT * FROM messages WHERE parent_message_id = ?",
                    id_bytes
                );

                let result: Vec<SqliteRow> = conn.fetch_all(query).await?;
                let messages = &mut vec![];

                result.iter().for_each(|row| {
                    let message = Message::from_row(row);
                    match message {
                        Ok(msg) => messages.push(msg),
                        _ => (),
                    }
                });

                if messages.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(mem::take(messages)))
                }
            }
        }
    }

    fn convert_uuid_to_bytes(uuid: &Uuid) -> &[u8] {
        let bytes = uuid.as_bytes();

        bytes as &[u8]
    }

    fn convert_to_uuid(bytes: Vec<u8>) -> Uuid {
        Uuid::from_bytes(Bytes::try_from(bytes.as_slice()).unwrap())
    }
}

#[graphql_object(context = Db)]
impl Message {
    pub fn id(&self) -> Option<Uuid> {
        self.id
    }

    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn is_reply(&self) -> bool {
        self.parent_message_id.is_some()
    }

    pub async fn parent_message(&self, ctx: &Db) -> Option<Message> {
        let connection = ctx.get().await.ok()?;
        let parent_message_id = self.parent_message_id?;
        Message::get(&parent_message_id, connection).await.ok()
    }

    pub async fn replies(&self, ctx: &Db) -> Option<Vec<Message>> {
        let connection = ctx.get().await.ok()?;
        self.get_replies(connection).await.ok()?
    }

    pub fn created_at(&self) -> Option<OffsetDateTime> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<OffsetDateTime> {
        self.updated_at
    }
}
