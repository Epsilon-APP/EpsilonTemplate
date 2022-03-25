use rocket::{Request, Response, response};
use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde_json::Value;

pub struct ApiSuccess {
    pub json: JsonValue,
    pub status: Status
}

impl ApiSuccess {
    pub fn default(message: &str) -> ApiSuccess {
        ApiSuccess {
            json: json!({
                "success": message
            }),
            status: Status::Ok
        }
    }

    pub fn data(data: Value) -> ApiSuccess {
        ApiSuccess {
            json: JsonValue::from(data),
            status: Status::Ok
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