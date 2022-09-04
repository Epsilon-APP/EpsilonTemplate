use std::fs::File;

use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::serde_json::json;

use crate::responses::api_success::ApiSuccess;
use crate::responses::file_upload::Upload;
use crate::{global, templates, ApiError};

use super::manager;

#[post("/<name>/push", data = "<data>")]
pub async fn push_map(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    let file = &mut data.upload;
    let map_file_path_str = format!("{}/{}.zip", global::MAPS_DIR, &name);

    file.persist_to(map_file_path_str)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The map has been pushed."))
}

#[delete("/<name>/delete")]
pub async fn delete(name: String) -> Result<ApiSuccess, ApiError> {
    if !manager::map_exist(&name) {
        return Err(ApiError::default("The map doesn't exist."));
    }

    let templates = templates::manager::get_templates()
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let mut template_using_map = false;

    for template in templates {
        if template.maps.contains(&name.to_string()) {
            template_using_map = true;
            break;
        }
    }

    if template_using_map {
        return Err(ApiError::new(
            "Some templates are using map.",
            Status::Conflict,
        ));
    }

    Ok(ApiSuccess::default("The map has been deleted."))
}

#[get("/")]
pub async fn get_maps() -> Result<ApiSuccess, ApiError> {
    let maps_dir = global::MAPS_DIR;
    let mut maps = Vec::new();

    let map_files = std::fs::read_dir(maps_dir)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?
        .filter_map(|file| file.ok())
        .filter(|file| file.path().is_file());

    for file in map_files {
        let path = file.path();
        let file_name_os_str = path.file_name().unwrap();
        let file_name_str = file_name_os_str.to_str().unwrap();

        if file_name_str.ends_with(".zip") {
            maps.push(file_name_str.replace(".zip", ""));
        }
    }

    Ok(ApiSuccess::data(json!(maps)))
}

#[get("/<name>/get")]
pub async fn get_map(name: String) -> Result<File, ApiError> {
    if !manager::map_exist(&name) {
        return Err(ApiError::default("The map doesn't exist."));
    }

    let map_file_path = manager::get_map_path(&name);

    Ok(File::open(map_file_path).unwrap())
}
