#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod manager;
mod parents;

mod api_error;
mod api_success;
mod templates;

use rocket::http::Status;
use rocket::routes;

#[get("/ping")]
fn ping() -> Status {
    Status::Ok
}

fn main() {
    rocket::ignite()
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
                templates::routes::zip,
            ],
        )
        .launch();
}
