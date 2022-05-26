use futures::executor::block_on;
use futures::TryStreamExt;
use reql::types::WriteStatus;
use reql::{cmd::connect::Options, r};

use super::cha;
use super::config;
use super::resources::RabbitStatus;

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRabbitData {
    id: String,
    status: RabbitStatus,
    body_color: String,
    patch_color: String,
    eye_color: String,
}

fn get_id(stat: &WriteStatus) -> Option<()> {
    if let Some(changes) = &stat.changes {
        if changes.len() > 0 {
            return Some(());
        }
    }
    None
}

async fn update_rabbit(id: &String) -> reql::Result<()> {
    let conn = r
        .connect(Options::new().port(config::RETHINKDB_PORT))
        .await?;

    let body_color = cha::hash(id);
    let patch_color = cha::hash(&body_color);
    let eye_color = cha::hash(&patch_color);

    let update = UpdateRabbitData {
        id: id.clone(),
        status: RabbitStatus::birthed,
        body_color: body_color,
        patch_color: patch_color,
        eye_color: eye_color,
    };
    println!("update data {:?}", update);

    let mut query = r.db("test").table("testrabbits").update(update).run(&conn);

    if let Some(result) = query.try_next().await? {
        if let Some(()) = get_id(&result) {
            println!("Update success id={}", id);
        } else {
            println!("Update failed id={}", id);
        }
    } else {
        println!("query error");
    }

    Ok(())
}

fn rabbit_generator(queue_name: &'static str) -> amiquip::Result<()> {
    use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
    use std::{thread, time};
    let mut connection = Connection::insecure_open(config::RABBITMQ_URL)?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(queue_name, QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("consumer spawned! topic={}", queue_name);

    for message in consumer.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                let body = String::from_utf8_lossy(&delivery.body);
                let id = format!("{}", body);
                consumer.ack(delivery)?;

                // Making rabbits takes time!
                let dur = time::Duration::from_millis(3_000);
                thread::sleep(dur);

                match block_on(update_rabbit(&id)) {
                    Ok(()) => (),
                    Err(err) => println!("failed to process id={} {:?}", id, err),
                };
            }
            other => {
                println!("consumer ended: {:?}", other);
                break;
            }
        }
    }
    Ok(())
}

pub fn init_rabbit_generator(queue_name: &'static str) -> amiquip::Result<()> {
    use std::{thread, time};

    thread::spawn(move || -> ! {
        let mut restarts = 0;
        loop {
            println!("consumer init: restarts={}", restarts);
            match rabbit_generator(queue_name) {
                Ok(()) => println!("consumer stopped, trying again..."),
                Err(e) => println!("consumer failed, trying again... {:?}", e),
            };

            restarts += 1;
            thread::sleep(time::Duration::from_millis(5_000));
        }
    });

    Ok(())
}
