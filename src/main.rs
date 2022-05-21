#![feature(plugin, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use]
extern crate serde_derive;

use futures::TryStreamExt;
use reql::{r, cmd::connect::Options};
use reql::types::ServerStatus;

// let session = r.connect(()).await?;

mod resources;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![
        index,
        rethink,
        resources::create_rabbit,
    ])
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
        Err(error) => "{\"error\":\"serde_error\"}".to_string(),
    }
}

/*
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
*/

// We are just going to print the JSON response for this example
fn handle(server_status: &ServerStatus) -> reql::Result<()> {
    println!("{}", serde_json::to_string(server_status)?);
    Ok(())
}

