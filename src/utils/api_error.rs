use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use rocket::{response, Request, Response};

#[derive(Debug)]
pub struct ApiError {
    pub json: Value,
    pub status: Status,
}

impl ApiError {
    pub fn new(message: &str, status: Status) -> ApiError {
        ApiError {
            json: json!({ "error": message }),
            status,
        }
    }

    pub fn default(message: &str) -> ApiError {
        ApiError {
            json: json!({ "error": message }),
            status: Status::InternalServerError,
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}
