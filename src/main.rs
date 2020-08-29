#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

mod ghidra;

use rocket::{
    http::RawStr, post,
    request::{Form, FromForm, FromFormValue},
    routes, State,
};
use rocket_contrib::serve::{crate_relative, StaticFiles};
use ghidra::GhidraServer;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
struct NonEmptyStr<'r>(&'r str);

impl<'r> FromFormValue<'r> for NonEmptyStr<'r> {
    type Error = &'static str;

    fn from_form_value(form_value: &'r RawStr) -> Result<Self, Self::Error> {
        if form_value.is_empty() {
            Err("value can not be empty")
        } else {
            Ok(NonEmptyStr(form_value.as_str()))
        }
    }
}

#[derive(Debug, FromForm)]
struct Register<'r> {
    username: Result<NonEmptyStr<'r>, &'static str>,
    password: Result<NonEmptyStr<'r>, &'static str>,
}

#[post("/register", data = "<user>")]
fn register(ghidra: State<'_, GhidraServer>, user: Form<Register<'_>>) -> Result<String, String> {
    let (username, password) = {
        let Register { username, password } = user.into_inner();
        if username.is_err() {
            return Err("`username` can not be empty".to_string());
        }

        if password.is_err() {
            return Err("`password` can not be empty".to_string());
        }

        (username.unwrap(), password.unwrap())
    };

    Ok("".to_string())
}

fn rocket(ghidra: GhidraServer) -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![register])
        .mount("/", StaticFiles::from(crate_relative!("/static")))
        .manage(ghidra)
}

#[rocket::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;
    let path = std::env::var("GHIDRA_SERVER_DIRECTORY")
        .map_err(|_| "you have to set the `GHIDRA_SERVER_DIRECTORY` env var")?;
    let ghidra = GhidraServer::from_dir(path.into())?;

    Ok(rocket(ghidra).launch().await?)
}
