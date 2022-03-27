use rocket::http::{ContentType, Status};
use rocket::response::Responder;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;
use rocket::{response, Request, Response};

pub struct ApiSuccess {
    pub json: Value,
    pub status: Status,
}

impl ApiSuccess {
    pub fn default(message: &str) -> ApiSuccess {
        ApiSuccess {
            json: json!({ "success": message }),
            status: Status::Ok,
        }
    }

    pub fn data(data: Value) -> ApiSuccess {
        ApiSuccess {
            json: data,
            status: Status::Ok,
        }
    }
}

#[rocket::async_trait]
impl<'r> Responder<'r, 'static> for ApiSuccess {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(self.json.respond_to(req).unwrap())
            .status(self.status)
            .header(ContentType::JSON)
            .ok()
    }
}

// impl<'r> Responder<'r> for ApiSuccess {
//     fn respond_to(self, req: &Request) -> response::Result<'r> {
//         Response::build_from(self.json.respond_to(req).unwrap())
//             // .status(self.status)
//             .header(ContentType::JSON)
//             .ok()
//     }
// }
