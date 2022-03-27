use crate::parents::parent::Parent;
use crate::utils::api_error::ApiError;
use crate::utils::api_success::ApiSuccess;
use crate::utils::file_upload::Upload;
use crate::{manager, Status};
use rocket::form::Form;
use rocket::serde::json::{serde_json, Json};
use std::fs::File;

fn init_dirs(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(manager::get_parent_plugins_path(name))
}

#[post("/create", data = "<data>")]
pub async fn create(data: Json<Parent>) -> Result<ApiSuccess, ApiError> {
    let parent = data.into_inner();
    let name = &parent.name;

    if manager::parent_exist(name) {
        return Err(ApiError::new("The parent already exist.", Status::Conflict));
    }

    init_dirs(name).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let parent_file_path_str = manager::get_parent_file_path(name);
    let parent_file = File::create(&parent_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(parent_file, &parent)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The parent has been created."))
}

#[put("/<name>/update", data = "<data>")]
pub async fn update(name: String, data: Json<Parent>) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    let parent_path = manager::get_parent_path(&name);
    let parent = data.into_inner();

    let new_name = &parent.name;
    let new_parent_path = manager::get_parent_path(new_name);

    std::fs::rename(parent_path, new_parent_path)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let new_parent_file_path_str = manager::get_parent_file_path(new_name);
    let new_parent_file = File::create(&new_parent_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(new_parent_file, &parent)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The parent has been updated."))
}

#[post("/<name>/plugins/push", data = "<data>")]
pub async fn push_plugin(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    let file = &mut data.upload;
    let file_name = file.name().unwrap();

    let plugin_path_str = manager::get_parent_plugins_path(&name);
    let plugin_file_path = format!("{}/{}.jar", plugin_path_str, file_name);

    file.persist_to(plugin_file_path)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The plugin has been pushed."))
}
//
#[post("/<name>/main/push", data = "<data>")]
pub async fn push_file(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    let file = &mut data.upload;
    let file_name = file.name().unwrap();
    let file_path = file.path().unwrap();
    let file_extension_os_str = file_path.extension().unwrap();
    let file_extension = file_extension_os_str.to_str().unwrap();

    let parent_path_str = manager::get_parent_path(&name);
    let new_file_path = format!("{}/{}.{}", parent_path_str, file_name, file_extension);

    file.persist_to(new_file_path)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The file has been pushed."))
}
