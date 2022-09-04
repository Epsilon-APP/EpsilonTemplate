use std::fs::File;
use std::io::Error;
use std::path::Path;

use rocket::serde::json::serde_json;

use crate::parents::parent::Parent;
use crate::templates::template::Template;
use crate::{global, parents};

pub fn get_templates() -> Result<Vec<Template>, Error> {
    let mut templates = Vec::new();
    let template_directories = std::fs::read_dir(global::TEMPLATES_DIR)?
        .filter_map(|dir| dir.ok())
        .filter(|dir| dir.path().is_dir());

    for dir in template_directories {
        let directory_path = dir.path();
        let directory_name_os_str = directory_path.file_name().unwrap();
        let directory_name = directory_name_os_str.to_str().unwrap();
        let current_template_result = get_template_obj(directory_name);

        if let Ok(..) = current_template_result {
            let mut current_template = current_template_result.unwrap();
            let current_template_parent = get_template_parent_obj(&current_template)?;

            current_template.t = Some(current_template_parent.t);

            templates.push(current_template);
        }
    }

    Ok(templates)
}

pub fn get_template_obj(name: &str) -> Result<Template, Error> {
    let details_file_path_str = get_details_file_path(name);
    let file = File::open(&details_file_path_str)?;

    Ok(serde_json::from_reader(&file)?)
}

pub fn get_template_parent_obj(template: &Template) -> Result<Parent, Error> {
    let parent_file_path_str = parents::manager::get_parent_file_path(&template.parent);
    let file = File::open(&parent_file_path_str)?;

    Ok(serde_json::from_reader(&file)?)
}

pub fn get_template_path(name: &str) -> String {
    format!("{}/{}", global::TEMPLATES_DIR, name)
}

pub fn get_template_plugins_path(name: &str) -> String {
    format!("{}/plugins", get_template_path(name))
}

pub fn get_details_file_path(name: &str) -> String {
    format!("{}/details.epsilon", get_template_path(name))
}

pub fn template_exist(name: &str) -> bool {
    let parent_file_path_str = &get_details_file_path(name);
    let parent_file_path = Path::new(parent_file_path_str);

    parent_file_path.exists()
}
