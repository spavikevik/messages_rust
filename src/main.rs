use rocket::response::content::RawHtml;
use rocket::{routes, State};

use crate::db::Db;
use crate::schema::{create_schema, Schema};

mod db;
mod message;
mod password_util;
mod schema;
mod user;

#[rocket::get("/graphiql")]
fn graphiql() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", None)
}

#[rocket::get("/playground")]
fn playground() -> RawHtml<String> {
    juniper_rocket::playground_source("/graphql", None)
}

#[rocket::get("/graphql?<request..>")]
async fn get_graphql(
    db: &State<Db>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, db).await
}

#[rocket::post("/graphql", data = "<request>")]
async fn post_graphql(
    db: &State<Db>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, db).await
}

#[rocket::main]
async fn main() {
    dotenvy::dotenv().expect("Couldn't read .env file");

    _ = rocket::build()
        .manage(Db::new())
        .manage(create_schema())
        .mount(
            "/",
            routes![graphiql, playground, get_graphql, post_graphql],
        )
        .launch()
        .await
        .expect("server to launch")
}
