#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;

use futures::TryStreamExt;
use reql::{r, cmd::connect::Options};
use reql::types::ServerStatus;
use rocket::fs::FileServer;

mod resources;

#[get("/rethink")]
async fn rethink() -> String {
    match tryrethink2().await {
        Ok(s) => s,
        Err(error) => {
            println!("error: {:?}", error);
            "error".to_string()
        },
    }
}

#[get("/rabbitmq")]
fn rabbitmq() -> String {
    match tryrabbitmq() {
        Ok(()) => "ok".to_string(),
        Err(err) => {
           println!("error: {:?}", err);
           "error".to_string()
        },
    }
}

#[get("/redis/getfoo")]
fn redis_getfoo() -> String {
    match redis_get() {
        Ok(s) => s,
        Err(err) => {
           println!("error: {:?}", err);
           "error".to_string()
        },
    }
}

#[get("/redis/setfoo")]
fn redis_setfoo() -> String {
    match redis_set() {
        Ok(()) => "OK".to_string(),
        Err(err) => {
           println!("error: {:?}", err);
           "error".to_string()
        },
    }
}

#[launch]
fn rocket() -> _ {
    use amiquip::{Connection};
    let mut connection = match Connection::insecure_open("amqp://guest:guest@localhost:55006") {
        Ok(c) => c,
        Err(err) => panic!("failed to connect: {:?}", err),
    };

    match rabbitmq_consume(&mut connection) {
        Ok(()) => println!("consumer started..."),
        Err(err) => println!("consumer fail: {:?}", err),
    };

    rocket::build()
    .manage(connection)
    .mount("/", FileServer::from("./static"))
    .mount("/", routes![
        rethink,
        rabbitmq,
        redis_getfoo,
        redis_setfoo,
        resources::create_rabbit,
        resources::create_rabbit2,
        resources::create_rabbit3,
        resources::get_rabbit,
        resources::get_rabbit3,
    ])
}

use amiquip::{Connection};
fn tryrabbitmq() -> amiquip::Result<()> {
    use amiquip::{Exchange, Publish};
    use std::time::{SystemTime};

    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:55006")?;
    // Open a channel - None says let the library choose the channel ID.
    let channel = connection.open_channel(None)?;

    // Get a handle to the direct exchange on our channel.
    let exchange = Exchange::direct(&channel);

    // Publish a message to the "hello" queue.
    let msg = format!("hello {:?}", SystemTime::now());
    exchange.publish(Publish::new(msg.as_bytes(), "hello"))?;

    connection.close()
}

fn rabbitmq_consume(connection: &mut Connection) -> amiquip::Result<()> {
    use amiquip::{QueueDeclareOptions, ConsumerOptions, ConsumerMessage};
    use std::thread;

    //let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:55006")?;
    let channel = connection.open_channel(None)?;

    thread::spawn(move || -> amiquip::Result<()> {
        //println!("consumer spawned!");
        let queue = channel.queue_declare("hello", QueueDeclareOptions::default()).map_err(|e| {
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
    let conn = r.connect(
        Options::new().port(55001)
    ).await?;

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
    let conn = r.connect(
        Options::new().port(55001)
    ).await?;

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
    let client = redis::Client::open("redis://127.0.0.1:55009/")?;
    let mut con = client.get_connection()?;

    /* do something here */
    let _: () = con.set("foo", "bar")?;

    Ok(())
}

fn redis_get() -> redis::RedisResult<String> {
    let client = redis::Client::open("redis://127.0.0.1:55009/")?;
    let mut con = client.get_connection()?;

    /* do something here */
    let value: String = con.get("foo")?;

    Ok(value)
}
