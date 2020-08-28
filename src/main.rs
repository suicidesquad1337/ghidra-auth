#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

use rocket::{http::RawStr, launch, request::FromForm};

use rocket_contrib::serve::{crate_relative, StaticFiles};

#[derive(FromForm)]
struct UserLogin<'r> {
    username: &'r RawStr,
    password: &'r RawStr,
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", StaticFiles::from(crate_relative!("/static")))
}
