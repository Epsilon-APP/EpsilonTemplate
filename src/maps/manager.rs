use std::path::Path;

use crate::global;

pub fn get_map_path(name: &str) -> String {
    format!("{}/{}.zip", global::MAPS_DIR, name)
}

pub fn map_exist(name: &str) -> bool {
    let map_path_str = &get_map_path(name);
    let map_path = Path::new(map_path_str);

    map_path.exists()
}
