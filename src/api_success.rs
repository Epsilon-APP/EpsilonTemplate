use rocket::{Request, Response, response};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde_derive::Serialize;
use serde_derive::Deserialize;

#[derive(Serialize, Deserialize)]
pub struct ApiSuccess {
    pub json: JsonValue,
    // pub status: Status
}

impl ApiSuccess {
    pub fn default(message: &str) -> ApiSuccess {
        ApiSuccess {
            json: json!({
                "success": message
            }),
            // status: Status::Ok
        }
    }
}

impl<'r> Responder<'r> for ApiSuccess {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        Response::build_from(self.json.respond_to(req).unwrap())
            // .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}