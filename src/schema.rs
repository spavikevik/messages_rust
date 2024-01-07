use juniper::futures::TryStreamExt;
use juniper::{graphql_object, EmptySubscription, GraphQLInputObject, RootNode};
use rocket::http::hyper::body::HttpBody;
use rocket_db_pools::Pool;
use sqlx::Row;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::*;
use crate::message::Message;
use crate::password_util::PasswordUtil;
use crate::user::User;

#[derive(Clone, Copy, Debug)]
pub struct Query;

#[graphql_object(context = Db)]
impl Query {
    async fn user(
        #[graphql(context)] database: &Db,
        #[graphql(description = "id of the user")] id: Uuid,
    ) -> Option<User> {
        let connection = database.get().await.ok()?;
        User::get(&id, connection).await.ok()
    }

    async fn message(
        #[graphql(context)] database: &Db,
        #[graphql(description = "id of the message")] id: Uuid,
    ) -> Option<Message> {
        let connection = database.get().await.ok()?;
        Message::get(&id, connection).await.ok()
    }

    async fn messages(
        #[graphql(context)] database: &Db,
        #[graphql(description = "(optional) user id")] user_id: Option<Uuid>,
        #[graphql(description = "after datetime")] after: OffsetDateTime,
        #[graphql(description = "before datetime")] before: OffsetDateTime,
    ) -> Option<Vec<Message>> {
        let connection = database.get().await.ok()?;
        Message::get_by_time_range(user_id.as_ref(), (after, before), connection)
            .await
            .ok()?
    }
}

pub struct Mutate {
    password_util: PasswordUtil<'static>,
}

#[graphql_object(context = Db)]
impl Mutate {
    async fn create_user(
        &self,
        #[graphql(context)] database: &Db,
        input_user: InputUser,
    ) -> Option<User> {
        let connection = database.get().await.ok()?;

        let password_hash = self.password_util.hash(input_user.password)?;

        let new_user = User::new(
            input_user.display_name.as_str(),
            input_user.username.as_str(),
            password_hash.as_str(),
        );

        new_user.insert(connection).await.ok()
    }

    async fn create_message(
        &self,
        #[graphql(context)] database: &Db,
        input_message: InputMessage,
    ) -> Option<Message> {
        let connection = database.get().await.ok()?;

        let new_message = Message::new(
            input_message.user_id,
            input_message.content.as_str(),
            input_message.parent_message_id,
        );

        new_message.insert(connection).await.ok()
    }

    async fn update_message(
        &self,
        #[graphql(context)] database: &Db,
        input_message: InputMessage,
    ) -> Option<Message> {
        let connection = database.get().await.ok()?;

        Message::update(&input_message.user_id, input_message.content, connection)
            .await
            .ok()
    }

    async fn delete_message(
        #[graphql(context)] database: &Db,
        #[graphql(description = "id of the message to be deleted")] id: Uuid,
    ) -> Option<Message> {
        let connection = database.get().await.ok()?;

        Message::delete(id, connection).await.ok()
    }
}

#[derive(GraphQLInputObject)]
#[graphql(description = "User input object")]
pub struct InputUser {
    display_name: String,
    username: String,
    password: String,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Message input object")]
pub struct InputMessage {
    user_id: Uuid,
    content: String,
    parent_message_id: Option<Uuid>,
}

#[derive(GraphQLInputObject)]
#[graphql(description = "Message update input object")]
pub struct UpdateMessage {
    id: Uuid,
    content: String,
}

pub type Schema = RootNode<'static, Query, Mutate, EmptySubscription<Db>>;

pub fn create_schema() -> Schema {
    Schema::new(
        Query {},
        Mutate {
            password_util: PasswordUtil::new(),
        },
        EmptySubscription::new(),
    )
}
