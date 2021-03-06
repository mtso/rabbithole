#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;

use futures::TryStreamExt;
use reql::types::ServerStatus;
use reql::{cmd::connect::Options, r};
use rocket::fs::FileServer;

mod cha;
mod config;
mod resources;
mod workers;

#[get("/ping")]
fn ping() -> () {
    ()
}

#[launch]
fn rocket() -> _ {
    println!("Initializing version: 0.1.2");

    use amiquip::Connection;
    let connection = match Connection::insecure_open(config::RABBITMQ_URL) {
        Ok(c) => c,
        Err(err) => panic!("failed to connect: {:?}", err),
    };

    match workers::init_rabbit_generator("rabbit.updated") {
        Err(err) => panic!("failed to initialize generator: {:?}", err),
        _ => (),
    };

    rocket::build()
        .manage(connection)
        .mount("/", FileServer::from("./ui/build"))
        .mount(
            "/",
            routes![
                ping,
                //rethink,
                //rabbitmq,
                //redis_getfoo,
                //redis_setfoo,
                //resources::create_rabbit,
                //resources::create_rabbit2,
                resources::create_rabbit3,
                //resources::get_rabbit,
                resources::get_rabbit3,
            ],
        )
}

#[allow(dead_code)]
#[get("/rethink")]
async fn rethink() -> String {
    match tryrethink2().await {
        Ok(s) => s,
        Err(error) => {
            println!("error: {:?}", error);
            "error".to_string()
        }
    }
}

#[allow(dead_code)]
#[get("/rabbitmq")]
fn rabbitmq() -> String {
    match tryrabbitmq() {
        Ok(()) => "ok".to_string(),
        Err(err) => {
            println!("error: {:?}", err);
            "error".to_string()
        }
    }
}

#[allow(dead_code)]
#[get("/redis/getfoo")]
pub fn redis_getfoo() -> String {
    match redis_get() {
        Ok(s) => s,
        Err(err) => {
            println!("error: {:?}", err);
            "error".to_string()
        }
    }
}

#[allow(dead_code)]
#[get("/redis/setfoo")]
pub fn redis_setfoo() -> String {
    match redis_set() {
        Ok(()) => "OK".to_string(),
        Err(err) => {
            println!("error: {:?}", err);
            "error".to_string()
        }
    }
}

use amiquip::Connection;
fn tryrabbitmq() -> amiquip::Result<()> {
    use amiquip::{Exchange, Publish};
    use std::time::SystemTime;

    let mut connection = Connection::insecure_open(config::RABBITMQ_URL)?;
    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    // Publish a message to the "hello" queue.
    let msg = format!("hello {:?}", SystemTime::now());
    exchange.publish(Publish::new(msg.as_bytes(), "hello"))?;

    connection.close()
}

#[allow(dead_code)]
fn rabbitmq_consume(connection: &mut Connection) -> amiquip::Result<()> {
    use amiquip::{ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
    use std::thread;

    let channel = connection.open_channel(None)?;

    thread::spawn(move || -> amiquip::Result<()> {
        let queue = channel
            .queue_declare("rabbitupdated", QueueDeclareOptions::default())
            .map_err(|e| {
                println!("error: {:?}", e);
                e
            })?;
        let consumer = queue.consume(ConsumerOptions::default()).map_err(|e| {
            println!("error: {:?}", e);
            e
        })?;

        println!("consumer spawned!");
        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    println!("Received [{}]", body);
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("Consumer ended: {:?}", other);
                    break;
                }
            }

            //println!("got message!: {:?}", message);
        }

        println!("consumer ending!");
        Ok(())
    });

    Ok(())
}

async fn tryrethink2() -> reql::Result<String> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let mut query = r.db("rethinkdb").table("server_status").run(&conn);

    if let Some(server_status) = query.try_next().await? {
        let res = handle2(&server_status);
        Ok(res)
    } else {
        Ok("error".to_string())
    }
}

fn handle2(server_status: &ServerStatus) -> String {
    match serde_json::to_string(server_status) {
        Ok(s) => s,
        Err(_error) => "{\"error\":\"serde_error\"}".to_string(),
    }
}

#[allow(dead_code)]
async fn tryrethink() -> reql::Result<()> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let mut query = r.db("rethinkdb").table("server_status").run(&conn);

    if let Some(server_status) = query.try_next().await? {
        handle(&server_status)?;
    }
    Ok(())
}

#[allow(dead_code)]
// We are just going to print the JSON response for this example
fn handle(server_status: &ServerStatus) -> reql::Result<()> {
    println!("{}", serde_json::to_string(server_status)?);
    Ok(())
}

extern crate redis;
use redis::Commands;

fn redis_set() -> redis::RedisResult<()> {
    let client = redis::Client::open(config::REDIS_URL)?;
    let mut con = client.get_connection()?;

    /* do something here */
    let _: () = con.set("foo", "bar")?;

    Ok(())
}

fn redis_get() -> redis::RedisResult<String> {
    let client = redis::Client::open(config::REDIS_URL)?;
    let mut con = client.get_connection()?;

    /* do something here */
    let value: String = con.get("foo")?;

    Ok(value)
}
