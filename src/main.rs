#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use rocket::{
    http::RawStr,
    launch, post,
    request::{Form, FromForm},
    routes,
};
use rocket_contrib::serve::{crate_relative, StaticFiles};

#[derive(Debug, FromForm)]
struct Register<'r> {
    username: &'r RawStr,
    password: &'r RawStr,
}

#[post("/register", data = "<user>")]
fn register(user: Form<Register<'_>>) -> Result<String, String> {
    Ok(format!("successfully registered"))
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![register])
        .mount("/", StaticFiles::from(crate_relative!("/static")))
}
