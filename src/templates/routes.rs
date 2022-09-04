use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::{serde_json, Json};
use rocket::State;
use std::fs::File;
use std::path::Path;
use zip::ZipWriter;

use crate::responses::api_error::ApiError;
use crate::responses::api_success::ApiSuccess;
use crate::responses::file_upload::Upload;
use crate::templates::template::Template;
use crate::{global, parents, Config, maps};

use super::{manager, utils};

fn init_dirs(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(manager::get_template_plugins_path(name))
}

#[get("/")]
pub async fn get_templates() -> Result<ApiSuccess, ApiError> {
    let templates =
        manager::get_templates().map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::data(json!(templates)))
}

#[get("/<name>")]
pub async fn get_template(name: String) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::BadRequest,
        ));
    }

    let mut current_template = manager::get_template_obj(&name)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let current_template_parent = manager::get_template_parent_obj(&current_template)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    current_template.t = Some(current_template_parent.t);

    Ok(ApiSuccess::data(json!(current_template)))
}

#[post("/create", data = "<data>")]
pub async fn create(data: Json<Template>) -> Result<ApiSuccess, ApiError> {
    let template = data.into_inner();

    if !parents::manager::parent_exist(&template.parent) {
        return Err(ApiError::new(
            "The specified parent doesn't exist.",
            Status::BadRequest,
        ));
    }

    let template_name = &template.name;

    if manager::template_exist(template_name) {
        return Err(ApiError::new(
            "The template already exists.",
            Status::Conflict,
        ));
    }

    let maps_name = &template.maps;

    for map_name in maps_name {
        if !maps::manager::map_exist(map_name) {
            return Err(ApiError::new(
                "A specified map doesn't exist.",
                Status::BadRequest,
            ));
        }
    }

    init_dirs(template_name).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let details_file_path_str = manager::get_details_file_path(template_name);
    let details_file_path = Path::new(&details_file_path_str);
    let details_file = File::create(details_file_path)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(details_file, &template)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The template has been created."))
}

#[delete("/<name>/delete")]
pub async fn delete(name: String) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let template_path_str = manager::get_template_path(&name);

    std::fs::remove_dir_all(template_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The template has been deleted."))
}

#[put("/<name>/update", data = "<data>")]
pub async fn update(
    name: String,
    data: Json<Template>,
    config: &State<Config>,
) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let template = data.into_inner();
    let template_path_str = manager::get_template_path(&name);

    let new_name = &template.name;
    let new_template_path_str = manager::get_template_path(new_name);

    std::fs::rename(template_path_str, new_template_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let new_details_file_path_str = manager::get_details_file_path(new_name);
    let new_details_file = File::create(new_details_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(new_details_file, &template)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    utils::build_template_dockerfile(&template, config)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The template has been updated."))
}

#[post("/<name>/plugins/push", data = "<data>")]
pub async fn push_plugin(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let file = &mut data.upload;
    let raw_name = file.raw_name().unwrap();
    let file_name = raw_name.dangerous_unsafe_unsanitized_raw();

    let plugin_path_str = manager::get_template_plugins_path(&name);
    let plugin_file_path = format!("{}/{}", plugin_path_str, file_name);

    file.persist_to(plugin_file_path)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The plugin has been pushed."))
}

#[post("/<name>/main/push", data = "<data>")]
pub async fn push_file(name: String, mut data: Form<Upload<'_>>) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let file = &mut data.upload;
    let raw_name = file.raw_name().unwrap();
    let file_name = raw_name.dangerous_unsafe_unsanitized_raw();

    let template_path_str = manager::get_template_path(&name);
    let new_file_path = format!("{}/{}", template_path_str, file_name);

    file.persist_to(new_file_path)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The file has been pushed."))
}

#[get("/<name>/zip")]
pub async fn to_zip(name: String) -> Result<File, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let template_path = manager::get_template_path(&name);
    let template = manager::get_template_obj(&name)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;
    let template_parent_name = template.parent;

    if !parents::manager::parent_exist(&template_parent_name) {
        return Err(ApiError::new(
            "The template's parent doesn't exist.",
            Status::NotFound,
        ));
    }

    let tmp_path_str = format!("{}/{}", global::TMP_DIR, name);
    let template_parent_path = parents::manager::get_parent_path(&template_parent_name);

    std::fs::create_dir_all(&tmp_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let parent_paths = glob::glob(&format!("{}/**/*", template_parent_path)).unwrap();
    let template_paths = glob::glob(&format!("{}/**/*", template_path)).unwrap();

    let zip_file_path = format!("{}/{}.zip", tmp_path_str, name);

    let file =
        File::create(&zip_file_path).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let mut zip = ZipWriter::new(file);

    utils::write_paths_in_zip(&mut zip, parent_paths, template_parent_name.as_str())
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    utils::write_paths_in_zip(&mut zip, template_paths, &name)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    zip.finish()
        .map_err(|_err| ApiError::default("An error occurred on finish writing zip file."))?;

    Ok(File::open(&zip_file_path).unwrap())
}

#[post("/<name>/build")]
pub async fn build(name: String, config: &State<Config>) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let current_template = manager::get_template_obj(&name)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    utils::build_template_dockerfile(&current_template, config)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default(
        "The image has been built on the registry.",
    ))
}
