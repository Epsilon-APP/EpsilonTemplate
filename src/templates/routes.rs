use crate::api_error::ApiError;
use crate::api_success::ApiSuccess;
use crate::manager;
use crate::manager::get_parent_path;
use crate::templates::template::Template;
use glob::Paths;
use rocket::http::{ContentType, Status};
use rocket::response::NamedFile;
use rocket_contrib::json::Json;
use rocket_upload::MultipartDatas;
use serde_json::json;
use std::fs::{File, OpenOptions, Permissions};
use std::io::{BufReader, Error, Read, Write};
use std::path::{Path, PathBuf};

use zip::write::FileOptions;
use zip::ZipWriter;

fn init_dirs(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(manager::get_template_path(name))
}

fn get_template_obj(name: &str) -> Result<Template, Error> {
    let details_file_path_str = &manager::get_details_file_path(name);
    let file = File::open(details_file_path_str)?;

    Ok(serde_json::from_reader(&file)?)
}

fn write_paths_in_zip(
    zip: &mut ZipWriter<File>,
    paths: Paths,
    prefix_path: &str,
) -> Result<(), Error> {
    for glob_result in paths {
        let path = glob_result.unwrap();
        let path_without_prefix: PathBuf = path
            .iter()
            .skip_while(|s| *s != prefix_path)
            .skip(1)
            .collect();

        if path.is_dir() {
            zip.add_directory(
                path_without_prefix.to_str().unwrap(),
                FileOptions::default(),
            )?
        } else if path.is_file() {
            let mut file = File::open(&path)?;
            let mut buffer = Vec::new();

            file.read_to_end(&mut buffer)?;

            print!("{}", buffer.len());

            zip.start_file(
                path_without_prefix.to_str().unwrap(),
                FileOptions::default(),
            )?;

            zip.write_all(&buffer)?
        }
    }

    Ok(())
}

#[get("/")]
pub fn get_templates() -> Result<ApiSuccess, ApiError> {
    let mut templates: Vec<Template> = Vec::new();

    let template_directories = std::fs::read_dir(manager::TEMPLATES_DIR)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?
        .filter_map(|dir| dir.ok())
        .filter(|dir| dir.path().is_dir());

    for dir in template_directories {
        let directory_path = dir.path();
        let directory_name_os_str = directory_path.file_name().unwrap();
        let directory_name = directory_name_os_str.to_str().unwrap();
        let current_template_result = get_template_obj(directory_name);

        if let Ok(..) = current_template_result {
            templates.push(current_template_result.unwrap());
        }
    }

    Ok(ApiSuccess::data(json!(templates)))
}

#[get("/<name>")]
pub fn get_template(name: String) -> Result<ApiSuccess, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::BadRequest,
        ));
    }

    let current_template =
        get_template_obj(&name).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::data(json!(current_template)))
}

#[post("/create", data = "<data>")]
pub fn create(data: Json<Template>) -> Result<ApiSuccess, ApiError> {
    let template = data.into_inner();

    if !manager::parent_exist(&template.parent) {
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

    init_dirs(template_name).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let details_file_path_str = manager::get_details_file_path(template_name);
    let details_file_path = Path::new(&details_file_path_str);
    let details_file = File::create(details_file_path)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(details_file, &template)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The template has been created."))
}

#[put("/<name>/update", data = "<data>")]
pub fn update(name: String, data: Json<Template>) -> Result<ApiSuccess, ApiError> {
    let template = data.into_inner();
    let template_path_str = manager::get_template_path(&name);

    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let new_name = &template.name;
    let new_template_path_str = manager::get_template_path(new_name);

    std::fs::rename(template_path_str, new_template_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let new_details_file_path_str = manager::get_details_file_path(new_name);
    let new_details_file = File::create(new_details_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    serde_json::to_writer_pretty(new_details_file, &template)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    Ok(ApiSuccess::default("The template has been updated."))
}

#[post("/<name>/plugins/push", data = "<data>")]
pub fn push_plugin(
    name: String,
    _content_type: &ContentType,
    data: MultipartDatas,
) -> Result<ApiSuccess, ApiError> {
    let plugin_path_str = manager::get_template_plugins_path(&name);

    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    if data.files.is_empty() {
        return Err(ApiError::new(
            "No plugin found in the body.",
            Status::BadRequest,
        ));
    }

    data.files[0].persist(Path::new(&plugin_path_str));

    Ok(ApiSuccess::default("The plugin has been pushed."))
}

#[post("/<name>/main/push", data = "<data>")]
pub fn push_file(
    name: String,
    _content_type: &ContentType,
    data: MultipartDatas,
) -> Result<ApiSuccess, ApiError> {
    let template_path = manager::get_template_path(&name);

    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    if data.files.is_empty() {
        return Err(ApiError::new(
            "No file found in the body.",
            Status::BadRequest,
        ));
    }

    data.files[0].persist(Path::new(&template_path));

    Ok(ApiSuccess::default("The file has been pushed."))
}

#[get("/<name>/zip")]
pub fn zip(name: String) -> Result<File, ApiError> {
    if !manager::template_exist(&name) {
        return Err(ApiError::new(
            "The template doesn't exist.",
            Status::NotFound,
        ));
    }

    let template_path = manager::get_template_path(&name);
    let template =
        get_template_obj(&name).map_err(|err| ApiError::default(err.to_string().as_str()))?;
    let template_parent_name = template.parent;

    if !manager::parent_exist(&template_parent_name) {
        return Err(ApiError::new(
            "The template's parent doesn't exist.",
            Status::NotFound,
        ));
    }

    let tmp_path_str = format!("./tmp/{}", name);
    let template_parent_path = get_parent_path(&template_parent_name);

    std::fs::create_dir_all(&tmp_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let parent_paths = glob::glob(&format!("{}/**/*", template_parent_path)).unwrap();
    let template_paths = glob::glob(&format!("{}/**/*", template_path)).unwrap();

    let zip_file_path = format!("{}/{}.zip", tmp_path_str, name);

    let file =
        File::create(&zip_file_path).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let mut zip = ZipWriter::new(file);

    write_paths_in_zip(&mut zip, parent_paths, template_parent_name.as_str())
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    write_paths_in_zip(&mut zip, template_paths, &name)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    zip.finish()
        .map_err(|_err| ApiError::default("An error occurred on finish writing zip file."))?;

    Ok(File::open(&zip_file_path).unwrap())
}
