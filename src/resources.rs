use rocket::serde::{Serialize, json::Json};

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

