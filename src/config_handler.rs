use std::{fs, io::Write, path::PathBuf};
use toml::Value;

// config handler
pub fn init_config() -> Value {
    let config_path = determine_config_path();
    let config_exists = config_path.exists();

    if !config_exists {
        copy_default_config(config_path.clone());
    }

    let config = read_config(config_path.clone());
    config.unwrap()
}

// Determine the path of the config file and make directories if required
fn determine_config_path() -> PathBuf {
    let mut config_directory = dirs::home_dir().unwrap();

    config_directory.push(".config");
    config_directory.push("squiid");
    config_directory.push("config.toml");

    let _ = fs::create_dir_all(config_directory.parent().unwrap());

    config_directory
}

// Copy default config file
// returns true if the config file exists, false if not
fn copy_default_config(config_path: PathBuf) -> bool {
    let mut file = fs::File::create(config_path.clone()).unwrap();
    let _ = file.write_all(include_bytes!("config.toml"));

    config_path.exists()
}

// read the config at the given path
fn read_config(config_path: PathBuf) -> Option<Value> {
    if config_path.exists() {
        let contents = fs::read_to_string(config_path).unwrap();
        let data: Value = toml::from_str(&contents).unwrap();
        Some(data)
    } else {
        None
    }
}

// write config file
fn write_config(config: Value, config_path: PathBuf) {
    let config_string = toml::to_string_pretty(&config).unwrap();

    let mut user_config_file = fs::File::create(config_path).unwrap();
    user_config_file
        .write_all(config_string.as_bytes())
        .unwrap();
}

// Function to update TOML config
pub fn update_user_config() -> Option<Value> {
    let config_path = determine_config_path();
    if !config_path.exists() {
        return None;
    }

    let system_config: Value = toml::from_str(include_str!("config.toml")).unwrap();

    let user_config_string = fs::read_to_string(config_path.clone()).unwrap();
    let mut user_config: Value = toml::from_str(&user_config_string).unwrap();

    // update the system config with the user's currently chosen config setup
    update_toml_values(&mut user_config, &system_config);

    write_config(user_config.clone(), config_path);

    Some(user_config)
}

// Recursive function to update TOML values
fn update_toml_values(user_config: &mut Value, system_config: &Value) {
    match (user_config, system_config) {
        (Value::Table(user_table), Value::Table(system_table)) => {
            // Update keys in user table with keys from system table
            for (key, system_value) in system_table {
                if !user_table.contains_key(key) {
                    // Key does not exist in user table, add it with system value
                    user_table.insert(key.clone(), system_value.clone());
                } else {
                    // Key exists in both user and system tables, recursively update values
                    if let Some(user_value) = user_table.get_mut(key) {
                        update_toml_values(user_value, system_value);
                    }
                }
            }
        }
        (Value::Array(user_array), Value::Array(system_array)) => {
            // Update elements in user array with elements from system array
            for (index, system_value) in system_array.iter().enumerate() {
                if let Some(user_value) = user_array.get_mut(index) {
                    update_toml_values(user_value, system_value);
                }
            }
        }
        _ => {}
    }
}

// TODO: wrapper for long .get() statements
