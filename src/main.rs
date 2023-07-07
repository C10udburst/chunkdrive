use global::Global;
use serde_yaml::from_reader;
use std::env::var;
use std::path::Path;

/* #region Modules */
mod blocks;
mod bucket;
mod encryption;
mod global;
mod inodes;
mod sources;
mod stored;

#[cfg(test)]
mod tests; // test utils and more tests
/* #endregion */

// these will be checked if CD_CONFIG_PATH is not set
const CONFIG_PATHS: [&str; 2] = ["./config.yml", "/etc/chunkdrive/config.yml"];

fn main() {
    // Find config file path
    let mut config_path = None;
    if let Ok(path) = var("CD_CONFIG_PATH") {
        config_path = Some(path);
    } else {
        for path in CONFIG_PATHS.iter() {
            if Path::new(path).exists() {
                config_path = Some(path.to_string());
                break;
            }
        }
    }

    // Load config file
    let file = std::fs::File::open(config_path.unwrap_or_else(||
        panic!("Could not find config file. Please set CD_CONFIG_PATH or place config.yml in one of the following locations: {:?}", CONFIG_PATHS))
    ).unwrap();
    let global: Global = from_reader(file).unwrap();

    println!("{:?}", global);
}