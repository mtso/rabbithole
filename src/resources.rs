use futures::TryStreamExt;
use reql::types::WriteStatus;
use reql::{cmd::connect::Options, r};
use rocket::serde::{json::Json, Serialize};

use super::config;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromForm)]
pub struct CreateRabbitRequest {
    name: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize)]
pub enum RabbitStatus {
    pending,
    birthed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRabbitData {
    name: String,
    created_at: DateTime<Utc>,
    status: RabbitStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rabbit {
    pub id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub status: RabbitStatus,
    pub name: String,
    pub body_color: Option<String>,
    pub patch_color: Option<String>,
    pub eye_color: Option<String>,
}

impl Clone for Rabbit {
    fn clone(&self) -> Rabbit {
        Rabbit{
            id: self.id.clone(),
            created_at: self.created_at.clone(),
            status: self.status.clone(),
            name: self.name.clone(),
            body_color: self.body_color.clone(),
            patch_color: self.patch_color.clone(),
            eye_color: self.eye_color.clone(),
        }
    }
}

impl Clone for RabbitStatus {
    fn clone(&self) -> RabbitStatus {
        match self {
            RabbitStatus::pending => RabbitStatus::pending,
            RabbitStatus::birthed => RabbitStatus::birthed,
        }
    }
}

#[allow(dead_code)]
#[post("/api/rabbits", data = "<request_data>")]
pub fn create_rabbit(request_data: Json<CreateRabbitRequest>) -> String {
    let data = request_data.into_inner();
    data.name
}

#[allow(dead_code)]
#[post("/api2/rabbits", data = "<request_data>")]
pub async fn create_rabbit2(request_data: Json<CreateRabbitRequest>) -> String {
    let data = request_data.into_inner();
    match save_rabbit(&data).await {
        Ok(s) => s,
        Err(_error) => "save error".to_string(),
    }
}

#[post("/api3/rabbits", data = "<request_data>")]
pub async fn create_rabbit3(request_data: Json<CreateRabbitRequest>) -> String {
    let data = request_data.into_inner();
    let new_rabbit = CreateRabbitData {
        name: data.name,
        created_at: Utc::now(),
        status: RabbitStatus::pending,
    };
    /*
    let rabbit = Rabbit{
        id: None,
        name: data.name,
        created_at: Utc::now(),
        body_color: None,
        patch_color: None,
        eye_color: None,
    };*/
    match save_rabbit2(&new_rabbit).await {
        Ok(s) => s,
        Err(err) => {
            println!("save error: {:?}", err);
            "save error".to_string()
        }
    }
}

#[allow(dead_code)]
#[get("/api/rabbits/<id>")]
pub async fn get_rabbit(id: String) -> String {
    match db_get_rabbit(id).await {
        Ok(s) => s,
        Err(error) => {
            println!("get error: {:?}", error);
            "get error".to_string()
        }
    }
}

#[get("/api3/rabbits/<id>")]
pub async fn get_rabbit3(id: String) -> String {
    match db_get_rabbit3(id).await {
        Ok(s) => s,
        Err(error) => {
            println!("get error: {:?}", error);
            "get error".to_string()
        }
    }
}

async fn save_rabbit2(rabbit: &CreateRabbitData) -> reql::Result<String> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let mut query = r.db("test").table("testrabbits").insert(rabbit).run(&conn);

    if let Some(write_status) = query.try_next().await? {
        if let Some(id) = get_id(&write_status) {
            match publish_rabbit(&id, "rabbit.updated") {
                Ok(()) => println!("published rabbit {}", id),
                Err(err) => println!("error publishing rabbit: {:?}", err),
            };

            let mut fetchq = r.db("test").table("testrabbits").get(id).run(&conn);

            if let Some(result) = fetchq.try_next().await? {
                match rabbit_to_json(&result) {
                    Ok(s) => Ok(s),
                    Err(err) => {
                        println!("json error: {:?}", err);
                        Ok("json error".to_string())
                    }
                }
            } else {
                Ok("error fetching".to_string())
            }
        } else {
            Ok("error getting id".to_string())
        }
    } else {
        Ok("error".to_string())
    }
}

fn rabbit_to_json(data: &Rabbit) -> serde_json::Result<String> {
    serde_json::to_string(data)
}

fn get_id(stat: &WriteStatus) -> Option<String> {
    if let Some(keys) = &stat.generated_keys {
        if keys.len() >= 1 {
            return keys.get(0).map(|uuid| uuid.to_hyphenated().to_string());
        }
    }
    None
}

async fn save_rabbit(rabbit: &CreateRabbitRequest) -> reql::Result<String> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let mut query = r.db("test").table("testrabbits").insert(rabbit).run(&conn);

    if let Some(write_status) = query.try_next().await? {
        let res = handle_write(&write_status);
        Ok(res)
    } else {
        Ok("error".to_string())
    }
}

async fn db_get_rabbit(id: String) -> reql::Result<String> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let mut query = r.db("test").table("testrabbits").get(id).run(&conn);

    if let Some(change) = query.try_next().await? {
        let res = handle_map(&change);
        Ok(res)
    } else {
        Ok("error".to_string())
    }
}

async fn db_get_rabbit3(id: String) -> reql::Result<String> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let mut query = r.db("test").table("testrabbits").get(id).run(&conn);

    if let Some(result) = query.try_next().await? {
        let rabbit: Rabbit = result;

        match rabbit.clone().status {
            RabbitStatus::pending => {
                if let Some(id) = rabbit.id.clone() {
                    if let Err(e) = publish_rabbit(&id, "rabbit.updated") {
                        println!("failed to publish rabbit_id={:?} {:?}", rabbit.id, e);
                    }
                }
            },
            _ => (),
        };

        match rabbit_to_json(&rabbit) {
            Ok(s) => Ok(s),
            Err(e) => {
                println!("error rendering json: {:?}", e);
                Ok("json error".to_string())
            }
        }
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

#[allow(dead_code)]
fn handle_query(status: &CreateRabbitRequest) -> String {
    status.name.clone()
}

fn handle_map(map: &HashMap<String, String>) -> String {
    match serde_json::to_string(map) {
        Ok(s) => s,
        Err(_err) => "json error".to_string(),
    }
}

use amiquip::Connection;
fn publish_rabbit(rabbit_id: &String, queue_name: &'static str) -> amiquip::Result<()> {
    use amiquip::{Exchange, Publish};

    let mut connection = Connection::insecure_open(config::RABBITMQ_URL)?;
    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    exchange.publish(Publish::new(rabbit_id.as_bytes(), queue_name))?;

    connection.close()
}
