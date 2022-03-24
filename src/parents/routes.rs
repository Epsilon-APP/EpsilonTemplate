use std::fs::File;
use std::path::Path;
use rocket::http::ContentType;
use rocket_contrib::json::Json;
use rocket_upload::MultipartDatas;
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
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let parent_file_path_str = &manager::get_parent_file_path(name);
    let parent_file = File::create(parent_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(parent_file, parent)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The parent has been created."))
}

#[put("/<name>/update", data = "<data>")]
pub fn update(name: String, data: Json<Parent>) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    let parent_path = &manager::get_parent_path(&name);
    let parent = &data.into_inner();

    let new_name = &parent.name;
    let new_parent_path = manager::get_parent_path(new_name);

    std::fs::rename(parent_path, new_parent_path)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let new_parent_file_path_str = &manager::get_parent_file_path(new_name);
    let new_parent_file = File::create(new_parent_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(new_parent_file, &parent)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The parent has been updated."))
}

#[post("/<name>/plugins/push", data = "<data>")]
pub fn push_plugin(name: String, _content_type: &ContentType, data: MultipartDatas) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    if data.files.is_empty() {
        return Err(ApiError::new("No plugin found in the body.", Status::BadRequest));
    }

    let plugin_path_str = &manager::get_parent_plugins_path(&name);
    let plugin_path = Path::new(&plugin_path_str);

    data.files[0].persist(plugin_path);

    Ok(ApiSuccess::default("The plugin has been pushed."))
}

#[post("/<name>/main/push", data = "<data>")]
pub fn push_file(name: String, _content_type: &ContentType, data: MultipartDatas) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    if data.files.is_empty() {
        return Err(ApiError::new("No file found in the body.", Status::BadRequest));
    }

    let parent_path_str = &manager::get_parent_path(&name);
    let parent_path = Path::new(parent_path_str);

    data.files[0].persist(parent_path);

    Ok(ApiSuccess::default("The file has been pushed.")
}