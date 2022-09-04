use std::fs::File;

use rocket::form::Form;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::{serde_json, Json};

use crate::parents::parent::Parent;
use crate::responses::api_error::ApiError;
use crate::responses::api_success::ApiSuccess;
use crate::responses::file_upload::Upload;
use crate::{global, templates, Status};

use super::manager;

fn init_dirs(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(manager::get_parent_plugins_path(name))
}

#[get("/")]
pub async fn get_parents() -> Result<ApiSuccess, ApiError> {
    let mut parents: Vec<Parent> = Vec::new();

    let parent_directories = std::fs::read_dir(global::PARENTS_DIR)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?
        .filter_map(|dir| dir.ok())
        .filter(|dir| dir.path().is_dir());

    for dir in parent_directories {
        let directory_path = dir.path();
        let directory_name_os_str = directory_path.file_name().unwrap();
        let directory_name = directory_name_os_str.to_str().unwrap();
        let current_parent_result = manager::get_parent_obj(directory_name);

        if let Ok(..) = current_parent_result {
            parents.push(current_parent_result.unwrap());
        }
    }

    Ok(ApiSuccess::data(json!(parents)))
}

#[get("/<name>")]
pub async fn get_parent(name: String) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new(
            "The parent doesn't exist.",
            Status::BadRequest,
        ));
    }

    let current_parent = manager::get_parent_obj(&name)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::data(json!(current_parent)))
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

#[delete("/<name>/delete")]
pub async fn delete(name: String) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    let templates = templates::manager::get_templates()
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let mut template_using_parent = false;

    for template in templates {
        if template.parent == name {
            template_using_parent = true;
            break;
        }
    }

    if template_using_parent {
        return Err(ApiError::new(
            "Some templates are using this parent.",
            Status::Conflict,
        ));
    }

    let parent_path = manager::get_parent_path(&name);

    std::fs::remove_dir_all(parent_path)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The parent has been deleted."))
}

#[post("/<name>/plugins/push", data = "<data>")]
pub async fn push_plugin(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    if !manager::parent_exist(&name) {
        return Err(ApiError::new("The parent doesn't exist.", Status::NotFound));
    }

    let file = &mut data.upload;
    let raw_name = file.raw_name().unwrap();
    let file_name = raw_name.dangerous_unsafe_unsanitized_raw();

    let plugin_path_str = manager::get_parent_plugins_path(&name);
    let plugin_file_path = format!("{}/{}", plugin_path_str, file_name);

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
    let raw_name = file.raw_name().unwrap();
    let file_name = raw_name.dangerous_unsafe_unsanitized_raw();

    let parent_path_str = manager::get_parent_path(&name);
    let new_file_path = format!("{}/{}", parent_path_str, file_name);

    file.persist_to(new_file_path)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The file has been pushed."))
}
