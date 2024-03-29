# Messages
A personal project as a practice of using Rust to develop a GraphQL (juniper) API.
It is a basic messaging API.

For a larger project with greater ambitions also written in Rust, please see [stereo](https://github.com/spavikevik/stereo).
Hopefully a README file will pop up there soon as well.

---
## Features
- User registration
- Message creation/modification/removal
- Message replies
---
## What I hope to work on later
- Error handling
- Logging
- Code quality (esp. idiomatic usage of Rust)
- Code repetition
- Database queries and indexing
- Probably lots of other things that I couldn't remember at the present moment
___
## Usage
### Create (sqlite) database
```shell
cargo sqlx db create
```
This will create a `db.sqlite` file in the root of this project.

### Run migrations
```shell
cargo sqlx migrate run
```
This will run the migrations defined under `/migrations`.

### Run server
```shell
cargo run
```
This will start up the server at `localhost:8000`

You can then use the GraphQL api available at `/graphql`.

Playground is provided at `/playground` and GraphiQL at `/grahpiql`.