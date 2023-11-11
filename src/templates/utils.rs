use bollard::auth::DockerCredentials;
use bollard::image::{BuildImageOptions, PushImageOptions, RemoveImageOptions};
use bollard::Docker;
use futures_util::StreamExt;
use glob::Paths;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::PathBuf;
use tar::Builder;
use zip::write::FileOptions;
use zip::ZipWriter;

use super::template::Template;
use crate::config::Config;
use crate::global;

pub async fn build_template_dockerfile(
    current_template: &Template,
    config: &Config,
) -> Result<(), Error> {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    let template_name = &current_template.name;

    let image_name = format!("{}/{}:latest", config.registry_host, template_name);
    let mut build_args = HashMap::new();

    build_args.insert("TEMPLATE_NAME", template_name.as_str());
    build_args.insert("DEFAULT_MAP_NAME", current_template.default_map.as_str());
    build_args.insert("API_HOST", config.api_host.as_str());

    let build_options = BuildImageOptions {
        dockerfile: "Dockerfile",
        t: &image_name,
        buildargs: build_args,
        rm: true,
        forcerm: true,
        pull: true,
        ..Default::default()
    };

    let dockerfile_path = "./data/Dockerfile";
    let mut dockerfile = File::open(dockerfile_path).unwrap();

    let tmp_path_str = format!("{}/{}", global::TMP_DIR, template_name);

    std::fs::create_dir_all(&tmp_path_str)?;

    let archive_file_path_str = format!("{}/{}.tar", tmp_path_str, template_name);
    let archive_file = File::create(&archive_file_path_str)?;

    let mut builder = Builder::new(archive_file);

    builder.append_file("Dockerfile", &mut dockerfile)?;

    builder.finish()?;

    let mut contents = Vec::new();
    File::open(&archive_file_path_str)?
        .read_to_end(&mut contents)
        .unwrap();

    let mut build_stream = docker.build_image(build_options, None, Some(contents.into()));

    while let Some(build_info) = build_stream.next().await {
        let build_info = build_info.map_err(|err| Error::new(ErrorKind::Other, err))?;

        if let Some(error) = build_info.error {
            return Err(Error::new(ErrorKind::Other, error));
        }
    }

    let credentials = DockerCredentials {
        username: Some(String::from(&config.registry_username)),
        password: Some(String::from(&config.registry_password)),
        ..Default::default()
    };

    let mut push_stream = docker.push_image(
        &image_name,
        None::<PushImageOptions<String>>,
        Some(credentials),
    );

    while let Some(push_info) = push_stream.next().await {
        let push_info = push_info.map_err(|err| Error::new(ErrorKind::Other, err))?;

        if let Some(error) = push_info.error {
            return Err(Error::new(ErrorKind::Other, error));
        }
    }

    let remove_image_options = RemoveImageOptions {
        force: true,
        ..Default::default()
    };

    let remove_image_stream = docker
        .remove_image(&image_name, Some(remove_image_options), None)
        .await
        .map_err(|err| Error::new(ErrorKind::Other, err))?;

    if remove_image_stream.is_empty() {
        return Err(Error::new(ErrorKind::Other, "Failed to remove image"));
    }

    Ok(())
}

pub fn write_paths_in_zip(
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
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        if path.is_dir() {
            zip.add_directory(path_without_prefix.to_str().unwrap(), options)?
        } else if path.is_file() {
            let mut file = File::open(&path)?;
            let mut buffer = Vec::new();

            file.read_to_end(&mut buffer)?;

            zip.start_file(path_without_prefix.to_str().unwrap(), options)?;
            zip.write_all(&buffer)?
        }
    }

    Ok(())
}
