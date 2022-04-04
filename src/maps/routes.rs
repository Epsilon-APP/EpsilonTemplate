use std::fs::File;

use rocket::form::Form;

use crate::utils::api_success::ApiSuccess;
use crate::utils::file_upload::Upload;
use crate::{manager, ApiError};

#[post("/<name>/push", data = "<data>")]
pub async fn push_map(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    let file = &mut data.upload;
    let map_file_path_str = format!("{}/{}.zip", manager::MAPS_DIR, &name);

    file.persist_to(map_file_path_str)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The map has been pushed."))
}

#[get("/<name>/get")]
pub async fn get_map(name: String) -> Result<File, ApiError> {
    if !manager::map_exist(&name) {
        return Err(ApiError::default("The map doesn't exist."));
    }

    let map_file_path = manager::get_map_path(&name);

    Ok(File::open(map_file_path).unwrap())
}
