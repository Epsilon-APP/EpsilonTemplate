use bollard::auth::DockerCredentials;
use bollard::image::{BuildImageOptions, PushImageOptions, RemoveImageOptions};
use bollard::Docker;
use futures_util::StreamExt;
use glob::Paths;
use rocket::form::Form;
use rocket::http::Status;
use rocket::serde::json::serde_json::json;
use rocket::serde::json::{serde_json, Json};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, Read, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::manager;
use crate::parents::parent::Parent;
use crate::templates::template::Template;
use crate::utils::api_error::ApiError;
use crate::utils::api_success::ApiSuccess;
use crate::utils::file_upload::Upload;

fn init_dirs(name: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(manager::get_template_plugins_path(name))
}

fn get_template_obj(name: &str) -> Result<Template, Error> {
    let details_file_path_str = manager::get_details_file_path(name);
    let file = File::open(&details_file_path_str)?;

    Ok(serde_json::from_reader(&file)?)
}

fn get_template_parent_obj(template: &Template) -> Result<Parent, Error> {
    let parent_file_path_str = manager::get_parent_file_path(&template.parent);
    let file = File::open(&parent_file_path_str)?;

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
pub async fn get_templates() -> Result<ApiSuccess, ApiError> {
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
            let mut current_template = current_template_result.unwrap();
            let current_template_parent = get_template_parent_obj(&current_template)
                .map_err(|err| ApiError::default(err.to_string().as_str()))?;

            current_template.t = Some(current_template_parent.t);

            templates.push(current_template);
        }
    }

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

    let mut current_template =
        get_template_obj(&name).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let current_template_parent = get_template_parent_obj(&current_template)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    current_template.t = Some(current_template_parent.t);

    Ok(ApiSuccess::data(json!(current_template)))
}

#[post("/create", data = "<data>")]
pub async fn create(data: Json<Template>) -> Result<ApiSuccess, ApiError> {
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
pub async fn update(name: String, data: Json<Template>) -> Result<ApiSuccess, ApiError> {
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
//
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
    let template_parent_path = manager::get_parent_path(&template_parent_name);

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

#[post("/<name>/build")]
pub async fn build(name: String) -> Result<ApiSuccess, ApiError> {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    let current_template =
        get_template_obj(&name).map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let resources = &current_template.resources;
    let minimum_resources = &resources.minimum;
    let maximum_resources = &resources.maximum;

    let min_ram = &minimum_resources.ram.to_string();
    let max_ram = &maximum_resources.ram.to_string();

    let registry_host =
        std::env::var("REGISTRY_HOST").unwrap_or_else(|_| "localhost:5000".to_string());
    let registry_username =
        std::env::var("REGISTRY_USERNAME").unwrap_or_else(|_| "admin".to_string());
    let registry_password =
        std::env::var("REGISTRY_PASSWORD").unwrap_or_else(|_| "admin".to_string());

    let api_host = std::env::var("API_HOST").unwrap_or_else(|_| "localhost:8000".to_string());

    let image_name = format!("{}/{}:latest", &registry_host, &name);
    let mut build_args = HashMap::new();

    build_args.insert("TEMPLATE_NAME", name.as_str());
    build_args.insert("API_HOST", api_host.as_str());

    build_args.insert("MIN_RAM", min_ram.as_str());
    build_args.insert("MAX_RAM", max_ram.as_str());

    let build_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: &image_name,
        buildargs: build_args,
        rm: true,
        forcerm: true,
        pull: true,
        ..Default::default()
    };

    let dockerfile_path = format!("{}/Dockerfile", manager::DATA_DIR);
    let mut dockerfile = File::open(&dockerfile_path).unwrap();

    let archive_file_path_str = format!("{}/archive.tar", manager::DATA_DIR);
    let archive_file = File::create(&archive_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let mut builder = Builder::new(archive_file);

    builder
        .append_file("Dockerfile", &mut dockerfile)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    builder
        .finish()
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    let mut contents = Vec::new();
    File::open(&archive_file_path_str)
        .map_err(|err| ApiError::default(err.to_string().as_str()))?
        .read_to_end(&mut contents)
        .unwrap();

    let mut build_stream = docker.build_image(build_options, None, Some(contents.into()));

    while let Some(build_info) = build_stream.next().await {
        let build_info = build_info.map_err(|err| ApiError::default(err.to_string().as_str()))?;

        if let Some(error) = build_info.error {
            return Err(ApiError::default(error.as_str()));
        }
    }

    let credentials = DockerCredentials {
        username: Some(registry_username),
        password: Some(registry_password),
        ..Default::default()
    };

    let mut push_stream = docker.push_image(
        &image_name,
        None::<PushImageOptions<String>>,
        Some(credentials),
    );

    while let Some(push_info) = push_stream.next().await {
        let push_info = push_info.map_err(|err| ApiError::default(err.to_string().as_str()))?;

        if let Some(error) = push_info.error {
            return Err(ApiError::default(error.as_str()));
        }
    }

    let remove_image_options = RemoveImageOptions {
        force: true,
        ..Default::default()
    };

    let remove_image_stream = docker
        .remove_image(&image_name, Some(remove_image_options), None)
        .await
        .map_err(|err| ApiError::default(err.to_string().as_str()))?;

    if remove_image_stream.is_empty() {
        return Err(ApiError::default("An error occurred on removing image."));
    }

    Ok(ApiSuccess::default(
        "The image has been built on the registry.",
    ))
}
