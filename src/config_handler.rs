use directories::{BaseDirs, ProjectDirs};
use std::{fs, io::Write, path::PathBuf};
use toml::Value;

/// Wrapper for the config
#[derive(Debug, Clone)]
pub struct Config {
    /// The toml config object
    pub config: Value,
}

impl From<Value> for Config {
    fn from(value: Value) -> Self {
        Self { config: value }
    }
}

impl Config {
    /// Get a section/key from the config
    pub fn get(&self, section: &str, key: &str) -> Option<&Value> {
        let section_value = self.config.get(section);
        if let Some(Value::Table(section_table)) = section_value {
            section_table.get(key)
        } else {
            None
        }
    }

    /// List sections in the config
    #[allow(dead_code)]
    pub fn list_sections(&self) -> Vec<&String> {
        match self.config.as_table() {
            Some(table) => table.keys().collect::<Vec<&String>>(),
            None => Vec::new(),
        }
    }

    /// List the keys within a section
    pub fn list_keys(&self, section: &str) -> Option<Vec<&String>> {
        let section_value = self.config.get(section);
        if let Some(Value::Table(section_table)) = section_value {
            Some(section_table.keys().collect::<Vec<&String>>())
        } else {
            None
        }
    }

    /// List the values within a section
    pub fn list_values(&self, section: &str) -> Option<Vec<&Value>> {
        let section_value = self.config.get(section);
        if let Some(Value::Table(section_table)) = section_value {
            Some(section_table.values().collect::<Vec<&Value>>())
        } else {
            None
        }
    }

    /// List the key, value pairs within a section
    /// returns a list of tuples
    /// [(key, value), (key, value)]
    pub fn list_items(&self, section: &str) -> Option<Vec<(&String, &Value)>> {
        let keys = self.list_keys(section);
        let values = self.list_values(section);

        if let (Some(key_list), Some(value_list)) = (keys, values) {
            let pairs: Vec<(&String, &Value)> = key_list
                .iter()
                .zip(value_list.iter())
                .map(|(k, v)| (&**k, *v))
                .collect();
            Some(pairs)
        } else {
            None
        }
    }

    /// Set a specific key in a specific section of the config
    pub fn set(&mut self, section: &str, key: &str, value: Value) {
        if let Value::Table(config) = &mut self.config {
            if let Some(Value::Table(section_config)) = config.get_mut(section) {
                section_config.insert(key.to_string(), value);
            }
        }
    }

    /// Create a new section in the config
    pub fn create_section(&mut self, section: &str) {
        if let Value::Table(config) = &mut self.config {
            config.insert(section.to_string(), Value::Table(toml::map::Map::new()));
        }
    }

    /// delete a section in the config
    pub fn delete_section(&mut self, section: &str) {
        if let Value::Table(config) = &mut self.config {
            config.remove(section);
        }
    }

    /// delete a key in a section of the config
    pub fn delete_key(&mut self, section: &str, key: &str) {
        if let Value::Table(config) = &mut self.config {
            if let Some(Value::Table(section_data)) = config.get_mut(section) {
                section_data.remove(key);
            }
        }
    }
}

/// Initialize the user config
/// Tests if the user config exists, and if not, it is created
pub fn init_config() {
    let config_path = determine_config_path();
    let config_exists = config_path.exists();

    if !config_exists {
        copy_default_config(config_path.clone());
    }
}

// TODO: document this somewhere
/// Determine the path of the config file and make directories if required
///
/// Linux: `~/.config/squiid/`
///
/// MacOS: `/Users/<NAME>/Library/Application Support/org.ImaginaryInfinity.Squiid/`
///
/// Windows: `C:\Users\<NAME>\AppData\Roaming\ImaginaryInfinity\Squiid\config`
///
/// Anything else: See Linux
fn determine_config_path() -> PathBuf {
    // try to determine correct config path
    let config_directory =
        if let Some(proj_dirs) = ProjectDirs::from("net", "ImaginaryInfinity", "Squiid") {
            let mut config_directory = proj_dirs.config_dir().to_path_buf();
            config_directory.push("config.toml");
            config_directory

        // couldn't determine config path, default to home directory .config folder
        } else {
            let home_dir = BaseDirs::new().unwrap().home_dir().to_owned();
            let config_directory: PathBuf = [
                home_dir.to_str().unwrap(),
                ".config",
                "squiid",
                "config.toml",
            ]
            .iter()
            .collect();
            config_directory
        };

    let _ = fs::create_dir_all(config_directory.parent().unwrap());

    config_directory
}

/// Copy default config file.
/// Returns true if the config file exists, false if not
fn copy_default_config(config_path: PathBuf) -> bool {
    let mut file = fs::File::create(config_path.clone()).unwrap();
    let _ = file.write_all(include_bytes!("config.toml"));

    config_path.exists()
}

/// Read the config at the given path
fn read_config(config_path: PathBuf) -> Option<Config> {
    if config_path.exists() {
        let contents = fs::read_to_string(config_path).unwrap();
        let data: Value = toml::from_str(&contents).unwrap();
        Some(Config { config: data })
    } else {
        None
    }
}

/// Write config file to a given path
fn write_config(config: Config, config_path: PathBuf) {
    let config_string = toml::to_string_pretty(&config.config).unwrap();

    let mut user_config_file = fs::File::create(config_path).unwrap();
    user_config_file
        .write_all(config_string.as_bytes())
        .unwrap();
}

/// Function to read the user config file and update it with any new values
/// that may have been added to the system config file
pub fn read_user_config() -> Option<Config> {
    let config_path = determine_config_path();
    let mut user_config = match read_config(config_path.clone()) {
        Some(config) => config,
        None => return None,
    };

    let system_config: Value = toml::from_str(include_str!("config.toml")).unwrap();

    // update the system config with the user's currently chosen config setup
    update_toml_values(&mut user_config.config, &system_config);

    write_config(user_config.clone(), config_path);

    Some(user_config)
}

/// Recursive function to update TOML values
/// shadows user_config onto system_config
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
