use std::fs::File;
use std::path::Path;
use rocket::http::hyper::StatusCode::InternalServerError;
use rocket_contrib::json::Json;
use crate::api_error::ApiError;
use crate::api_success::ApiSuccess;
use crate::{manager, Status};
use crate::parents::parent::Parent;

fn init_dirs(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(manager::get_parent_plugins_path(name))
}

#[post("/create", data = "<data>")]
pub fn create(data: Json<Parent>) -> Result<ApiSuccess, ApiError> {
    let parent = &data.into_inner();
    let name = &parent.name;

    if manager::parent_exist(name) {
        return Err(ApiError::new("The parent already exist.", Status::Conflict));
    }

    init_dirs(name)
        .map_err(|err| ApiError::new(err.to_string().as_str(), Status::InternalServerError))?;

    let parent_file_path_str = &manager::get_parent_file_path(name);
    let parent_file = File::create(parent_file_path_str)
        .map_err(|err| ApiError::new(err.to_string().as_str(), Status::InternalServerError))?;

    serde_json::to_writer_pretty(parent_file, parent)
        .map_err(|err| ApiError::new(err.to_string().as_str(), Status::InternalServerError))?;

    Ok(ApiSuccess::new("The parent has been created."))

    // let mkdir_result = init_dirs(name);
    //
    // match mkdir_result {
    //     Ok(_) => {
    //         let parent_file_path_str = &manager::get_parent_file_path(name);
    //         let parent_file_path = Path::new(parent_file_path_str);
    //         let writing_result = serde_json::to_writer_pretty(File::create(parent_file_path).unwrap(), &data.into_inner());
    //
    //         match writing_result {
    //             Ok(_) => {
    //                 ApiResponse::success("The parent has been created.", Status::Ok)
    //             }
    //             Err(error) => {
    //                 ApiResponse::error(error.to_string().as_str(), Status::InternalServerError)
    //             }
    //         }
    //     }
    //     Err(error) => {
    //         ApiResponse::error(error.to_string().as_str(), Status::InternalServerError)
    //     }
    // }
}