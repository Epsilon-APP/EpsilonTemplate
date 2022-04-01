#[macro_use]
extern crate rocket;

mod manager;
mod parents;
mod templates;
mod utils;

use crate::utils::api_error::ApiError;
use rocket::data::{Limits, ToByteUnit};
use rocket::http::Status;
use rocket::{routes, Request};

fn init_base_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(manager::PARENTS_DIR)?;
    std::fs::create_dir_all(manager::TEMPLATES_DIR)
}

#[get("/ping")]
fn ping() -> Status {
    Status::Ok
}

#[catch(default)]
fn default_catcher(status: Status, _request: &Request) -> ApiError {
    ApiError::new("An error is occurred with the server.", status)
}

#[launch]
fn rocket() -> _ {
    init_base_dirs().expect("Failed to create base directories");

    let limits = Limits::default()
        .limit("file", 100.megabytes())
        .limit("data-form", 100.megabytes());

    let config = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("limits", limits));

    rocket::custom(config)
        .register("/", catchers![default_catcher])
        .mount("/", routes![ping])
        .mount(
            "/parents",
            routes![
                parents::routes::create,
                parents::routes::update,
                parents::routes::push_plugin,
                parents::routes::push_file
            ],
        )
        .mount(
            "/templates",
            routes![
                templates::routes::get_template,
                templates::routes::get_templates,
                templates::routes::create,
                templates::routes::update,
                templates::routes::push_plugin,
                templates::routes::push_file,
                templates::routes::to_zip,
                templates::routes::build
            ],
        )
}
