#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use rocket::{launch, post, routes};
use rocket_contrib::{
    json,
    json::{Json, JsonValue},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Register {
    username: String,
    password: String,
}

#[post("/register", format = "json", data = "<user>")]
fn register(user: Json<Register>) -> JsonValue {
    json!({ "status": "ok" })
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![register])
}
