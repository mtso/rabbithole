use amiquip::{Connection};
use futures::TryStreamExt;
use futures::executor::block_on;
use reql::types::WriteStatus;
use reql::{r, cmd::connect::Options};

use super::resources::RabbitStatus;
use super::cha;

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
    let conn = r.connect(
        Options::new().port(55001)
    ).await?;

    let body_color = cha::hash(id);
    let patch_color = cha::hash(&body_color);
    let eye_color = cha::hash(&patch_color);

    let update = UpdateRabbitData{
        id: id.clone(),
        status: RabbitStatus::birthed,
        body_color: body_color,//String::from("#123123"),
        patch_color: patch_color, // String::from("#123123"),
        eye_color: eye_color, //String::from("#123123"),
    };
    println!("update data {:?}", update);

    let mut query = r.db("test").table("testrabbits")
        .update(update)
        .run(&conn);

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


pub fn init_rabbit_generator(connection: &mut Connection, queue_name: &'static str) -> amiquip::Result<()> {
    use amiquip::{QueueDeclareOptions, ConsumerOptions, ConsumerMessage};
    use std::thread;

    let channel = connection.open_channel(None)?;

    thread::spawn(move || -> amiquip::Result<()> {
        let queue = channel.queue_declare(queue_name, QueueDeclareOptions::default()).map_err(|e| {
            println!("error: {:?}", e);
            e
        })?;

        let consumer = queue.consume(ConsumerOptions::default()).map_err(|e| {
            println!("error: {:?}", e);
            e
        })?;

        println!("consumer spawned! topic={}", queue_name);

        for message in consumer.receiver().iter() {

        match message {
            ConsumerMessage::Delivery(delivery) => {

                let body = String::from_utf8_lossy(&delivery.body);
                let id = format!("{}", body);
                match block_on(update_rabbit(&id)) {
                    Ok(()) => (),
                    Err(err) => println!("failed to process id={}", id),
                };

                consumer.ack(delivery)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }

        }

        println!("consumer ending!");
        Ok(())
    });

    Ok(())
}


