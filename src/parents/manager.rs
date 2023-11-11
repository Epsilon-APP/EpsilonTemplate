use std::{fs::File, io::Error, path::Path};

use rocket::serde::json::serde_json;

use crate::global;

use super::parent::Parent;

pub fn get_parent_path(name: &str) -> String {
    format!("{}/{}", global::PARENTS_DIR, name)
}

pub fn get_parent_plugins_path(name: &str) -> String {
    format!("{}/plugins", get_parent_path(name))
}

pub fn get_parent_file_path(name: &str) -> String {
    format!("{}/parents.epsilon", get_parent_path(name))
}

pub fn parent_exist(name: &str) -> bool {
    let parent_file_path_str = &get_parent_file_path(name);
    let parent_file_path = Path::new(parent_file_path_str);

    parent_file_path.exists()
}

pub fn get_parent_obj(name: &str) -> Result<Parent, Error> {
    let parent_file_path_str = get_parent_file_path(name);
    let file = File::open(parent_file_path_str)?;

    Ok(serde_json::from_reader(&file)?)
}
