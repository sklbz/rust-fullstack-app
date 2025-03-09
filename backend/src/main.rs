#[macro_use]
extern crate rocket;

use rocket::execute;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::{State, http::Status, response::status::Custom};
use rocket_cors::{AllowedOrigins, CorsOptions};
use tokio_postgres::{Client, NoTls};

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: i32,
    name: String,
    hashed_password: String,
}

#[post("/api/users", data = "<user>")]
async fn add_user(user: Json<User>, conn: &State<Client>) -> Result<Json<User>, Custom<Status>> {
    execute_query(
        conn,
        "INSERT INTO users (name, hashed_password) VALUES ($1, $2) RETURNING id",
        &[&user.name, &user.hashed_password],
    )
    .await?;
    get_users(conn).await
}

#[get("/api/users")]
async fn get_users(conn: &State<Client>) -> Result<Json<Vec<User>>, Custom<String>> {
    get_users_from_db(conn).await.map(Json)
}

async fn get_users_from_db(client: &Client) -> Result<Vec<User>, Custom<String>> {
    let users = client
        .query("SELECT id, name, hashed_password FROM users", &[])
        .await
        .map_err(|e| {
            Custom(
                Status::InternalServerError,
                format!("Database error: {}", e),
            )
        })
        .iter()
        .map(|row| User {
            id: Some(row.get(0)),
            name: row.get(1),
            hashed_password: row.get(2),
        })
        .collect::<Vec<User>>();

    Ok(users)
}

#[put("/api/users/<id>", data = "<user>")]
async fn update_user(
    id: i32,
    user: Json<User>,
    conn: &State<Client>,
) -> Result<Json<User>, Custom<Status>> {
    execute_query(
        conn,
        "UPDATE users SET name = $1, hashed_password = $2 WHERE id = $3",
        &[&user.name, &user.hashed_password, &id],
    )
    .await?;

    get_users(conn).await
}

#[delete("/api/users/<id>")]
async fn delete_user(id: i32, conn: &State<Client>) -> Result<Status, Custom<String>> {
    execute_query(conn, "DELETE FROM users WHERE id = $1", &[&id]).await?;

    Ok(Status::NoContent)
}

async fn execute_query(
    client: &Client,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
) -> Result<u64, Custom<String>> {
    client.execute(query, params).await.map_err(|e| {
        Custom(
            Status::InternalServerError,
            format!("Database error: {}", e),
        )
    })
}

#[launch]
async fn rocket() -> _ {
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=postgres",
        NoTls,
    )
    .await
    .expect("Unable to connect to database");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Create the table if it doesn't exist
    execute_query(
        &client,
        "CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            hashed_password TEXT NOT NULL
        )",
        &[],
    )
    .await
    .expect("Unable to create table");

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Failed to create CORS options");

    rocket::build()
        .mount("/", routes![add_user, get_users, update_user, delete_user])
        .manage(client)
        .attach(cors)
}
