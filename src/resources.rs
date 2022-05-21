use rocket::serde::{Serialize, json::Json};
use reql::types::WriteStatus;
use futures::TryStreamExt;
use reql::{r, cmd::connect::Options};

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct CreateRabbitRequest {
    name: String,
}

#[post("/api/rabbits", data = "<request_data>")]
pub fn create_rabbit(
    request_data: Json<CreateRabbitRequest>,
) -> String {
    let data = request_data.into_inner();
    data.name
}

#[post("/api2/rabbits", data = "<request_data>")]
pub async fn create_rabbit2(
    request_data: Json<CreateRabbitRequest>,
) -> String {
    let data = request_data.into_inner();
    match save_rabbit(&data).await {
        Ok(s) => s,
        Err(_error) => "save error".to_string(),
    }
}

#[get("/api/rabbits/<id>")]
pub async fn get_rabbit(
    id: String,
) -> String {
    match db_get_rabbit(id).await {
        Ok(s) => s,
        Err(error) => {
            println!("get error: {:?}", error);
            "get error".to_string()
        }
    }
}

async fn save_rabbit(rabbit: &CreateRabbitRequest) -> reql::Result<String> {
    let conn = r.connect(
        Options::new().port(55001)
    ).await?;

    let mut query = r.db("test").table("testrabbits")
        .insert(rabbit)
        .run(&conn);

    if let Some(write_status) = query.try_next().await? {
        let res = handle_write(&write_status);
        Ok(res)
    } else {
        Ok("error".to_string())
    }
}

async fn db_get_rabbit(id: String) -> reql::Result<String> {
    let conn = r.connect(
        Options::new().port(55001)
    ).await?;

    let mut query = r.db("test").table("testrabbits")
        .get(id)
        .run(&conn);

    if let Some(change) = query.try_next().await? {
        let res = handle_map(&change);
        Ok(res)
    } else {
        Ok("error".to_string())
    }
}

fn handle_write(status: &WriteStatus) -> String {
    match serde_json::to_string(status) {
        Ok(s) => s,
        Err(_err) => "json error".to_string(),
    }
}

fn handle_query(status: &CreateRabbitRequest) -> String {
    status.name.clone()
}

fn handle_map(map: &HashMap<String, String>) -> String {
    match serde_json::to_string(map) {
        Ok(s) => s,
        Err(_err) => "json error".to_string(),
    }
}

