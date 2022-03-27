use std::path::Path;

// PARENTS

pub const PARENTS_DIR: &str = "./data/parents";
pub const TEMPLATES_DIR: &str = "./data/templates";
pub const DATA_DIR: &str = "./data";

pub fn get_parent_path(name: &str) -> String {
    format!("{}/{}", PARENTS_DIR, name)
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

// TEMPLATES

pub fn get_template_path(name: &str) -> String {
    format!("{}/{}", TEMPLATES_DIR, name)
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
