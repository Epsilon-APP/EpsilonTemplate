use std::fmt::{Display, Formatter};
use rocket::{Request, Response, response};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;

#[derive(Debug)]
pub struct ApiError {
    pub json: JsonValue,
    pub status: Status,
}

impl ApiError {
    pub fn new(message: &str, status: Status) -> ApiError {
        ApiError {
            json: json!({
                "error": message
            }),
            status
        }
    }
}

impl<'r> Responder<'r> for ApiError {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}