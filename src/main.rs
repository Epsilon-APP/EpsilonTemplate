#[macro_use]
extern crate rocket;

use rocket::data::{Limits, ToByteUnit};
use rocket::http::Status;
use rocket::{routes, Request};

use crate::config::Config;
use crate::responses::api_error::ApiError;

mod config;
mod global;
mod maps;
mod parents;
mod responses;
mod templates;

fn init_base_dirs() -> std::io::Result<()> {
    std::fs::create_dir_all(global::PARENTS_DIR)?;
    std::fs::create_dir_all(global::MAPS_DIR)?;
    std::fs::create_dir_all(global::DATA_TMP_FILES_DIR)?;
    std::fs::create_dir_all(global::TEMPLATES_DIR)
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

    let config = Config::new("admin", "admin", "localhost:5000", "localhost:8000");

    std::env::set_var("TMPDIR", global::DATA_TMP_FILES_DIR);

    let limits = Limits::default()
        .limit("file", 100.megabytes())
        .limit("data-form", 100.megabytes());

    let rocket_config = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("limits", limits));

    rocket::custom(rocket_config)
        .register("/", catchers![default_catcher])
        .manage(config)
        .mount("/", routes![ping])
        .mount(
            "/parents",
            routes![
                parents::routes::get_parent,
                parents::routes::get_parents,
                parents::routes::create,
                parents::routes::delete,
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
                templates::routes::delete,
                templates::routes::update,
                templates::routes::push_plugin,
                templates::routes::push_file,
                templates::routes::to_zip,
                templates::routes::build
            ],
        )
        .mount(
            "/maps",
            routes![
                maps::routes::delete,
                maps::routes::push_map,
                maps::routes::get_map,
                maps::routes::get_maps
            ],
        )
}
