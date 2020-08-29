#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

mod ghidra;

use ghidra::GhidraServer;
use rocket::{
    http::RawStr,
    post,
    request::{Form, FromForm, FromFormValue},
    routes, State,
};
use rocket_contrib::serve::{crate_relative, StaticFiles};

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
}

#[post("/register", data = "<user>")]
async fn register(
    ghidra: State<'_, GhidraServer>,
    user: Form<Register<'_>>,
) -> Result<String, String> {
    let username = {
        let Register { username } = user.into_inner();
        if let Ok(username) = username {
            username
        } else {
            return Err("`username` can not be empty".to_string());
        }
    };

    if let Err(_) = ghidra.add_user(username.0).await {
        Err("Failed to create your account.".to_string())
    } else {
        Ok("Successfully created your account.\
            Now login using `changeme` as the password and change your password"
            .to_string())
    }
}

fn rocket(ghidra: GhidraServer) -> rocket::Rocket {
    rocket::ignite()
        .mount("/", routes![register])
        .mount("/", StaticFiles::from(crate_relative!("/static")))
        .manage(ghidra)
}

#[rocket::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv()?;
    let path = std::env::var("GHIDRA_SERVER_DIRECTORY")
        .map_err(|_| "you have to set the `GHIDRA_SERVER_DIRECTORY` env var")?;
    let ghidra = GhidraServer::from_dir(path.into())?;

    Ok(rocket(ghidra).launch().await?)
}
