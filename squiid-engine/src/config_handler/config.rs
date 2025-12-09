use std::{fmt::Debug, path::PathBuf};

use toml::Value;

use crate::config_handler::{ConfigBackend, ConfigError};

/// Wrapper for the config
#[derive(Debug)]
pub struct Config {
    /// The toml config object
    config: Value,
    /// The persistance strategy
    backend: Box<dyn ConfigBackend>,
}

impl Config {
    #[cfg(any(unix, windows))]
    pub fn new() -> Self {
        use crate::config_handler::backend::fs::FsBackend;

        Self::new_with_backend(Box::new(FsBackend::new()))
    }

    pub fn new_with_backend(backend: Box<dyn ConfigBackend>) -> Self {
        let mut config = Self {
            backend,
            config: Value::Table(Default::default()),
        };
        config.load();
        config
    }

    pub fn config_directory(&self) -> Option<PathBuf> {
        self.backend.config_directory()
    }

    /// Set the persistence backend of the config.
    ///
    /// By default on Unix and Windows systems, the config is automatically set up to save to disk
    /// in the appropriate configuration file location. However on other targets such as WASM, the
    /// persistence backend (a function implementing [`crate::config_handler::ConfigBackend`])
    /// must be explicitly set with this function.
    ///
    /// ## NOTE: After setting this, you may want to call the `load()` method to reload the config
    /// with the new backend.
    ///
    /// # Arguments
    ///
    /// * `backend` - the configuration backend to use
    ///
    /// # Panics
    ///
    /// Panics if the engine mutex cannot be locked.
    pub fn set_backend(&mut self, backend: Box<dyn ConfigBackend>) {
        self.backend = backend;
    }

    pub fn load(&mut self) {
        if let Some(content) = self.backend.load()
            && let Ok(val) = toml::from_str(&content)
        {
            self.config = val;
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let content = toml::to_string_pretty(&self.config).unwrap_or_default();
        self.backend.save(&content)
    }

    /// Get a section/key from the config
    #[allow(dead_code)]
    pub fn get_key(&self, section: &str, key: &str) -> Result<Value, ConfigError> {
        let section_value = self.config.get(section);
        if let Some(Value::Table(section_table)) = section_value {
            match section_table.get(key) {
                Some(value) => Ok(value.clone()),
                None => Err(ConfigError::MissingKey {
                    section: section.to_owned(),
                    key: key.to_owned(),
                }),
            }
        } else {
            Err(ConfigError::MissingSection(section.to_owned()))
        }
    }

    /// List sections in the config
    #[allow(dead_code)]
    pub fn list_sections(&self) -> Vec<String> {
        match self.config.as_table() {
            Some(table) => table.keys().map(|s| s.to_owned()).collect(),
            None => Vec::new(),
        }
    }

    /// List the keys within a section
    #[allow(dead_code)]
    pub fn list_keys(&self, section: &str) -> Result<Vec<String>, ConfigError> {
        let section_value = self.config.get(section);
        if let Some(Value::Table(section_table)) = section_value {
            Ok(section_table.keys().map(|s| s.to_owned()).collect())
        } else {
            Err(ConfigError::MissingSection(section.to_owned()))
        }
    }

    #[allow(dead_code)]
    pub fn contains_key(&self, section: &str, key: &str) -> Result<bool, ConfigError> {
        if let Some(Value::Table(section_table)) = self.config.get(section) {
            Ok(section_table.keys().filter(|&s| s == key).count() > 0)
        } else {
            Err(ConfigError::MissingSection(section.to_owned()))
        }
    }

    /// List the values within a section
    #[allow(dead_code)]
    pub fn list_values(&self, section: &str) -> Result<Vec<Value>, ConfigError> {
        let section_value = self.config.get(section);
        if let Some(Value::Table(section_table)) = section_value {
            Ok(section_table.values().map(|v| v.to_owned()).collect())
        } else {
            Err(ConfigError::MissingSection(section.to_owned()))
        }
    }

    /// List the key, value pairs within a section
    /// returns a list of tuples
    /// [(key, value), (key, value)]
    #[allow(dead_code)]
    pub fn list_items(&self, section: &str) -> Result<Vec<(String, Value)>, ConfigError> {
        let keys = self.list_keys(section);
        let values = self.list_values(section);

        if let (Ok(key_list), Ok(value_list)) = (keys, values) {
            let pairs: Vec<(String, Value)> = key_list
                .iter()
                .zip(value_list.iter())
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect();
            Ok(pairs)
        } else {
            Err(ConfigError::MissingSection(section.to_owned()))
        }
    }

    /// Set a specific key in a specific section of the config
    #[allow(dead_code)]
    pub fn set_key(&mut self, section: &str, key: &str, value: Value) -> Result<(), ConfigError> {
        if let Value::Table(config) = &mut self.config {
            if let Some(Value::Table(section_config)) = config.get_mut(section) {
                section_config.insert(key.to_string(), value);
                Ok(())
            } else {
                Err(ConfigError::MissingSection(section.to_owned()))
            }
        } else {
            Err(ConfigError::MalformedConfig)
        }
    }

    /// Create a new section in the config
    #[allow(dead_code)]
    pub fn create_section(&mut self, section: &str) -> Result<(), ConfigError> {
        if let Value::Table(config) = &mut self.config {
            config.insert(section.to_string(), Value::Table(toml::map::Map::new()));
            Ok(())
        } else {
            Err(ConfigError::MalformedConfig)
        }
    }

    /// delete a section in the config
    #[allow(dead_code)]
    pub fn delete_section(&mut self, section: &str) -> Result<(), ConfigError> {
        if let Value::Table(config) = &mut self.config {
            config.remove(section);
            Ok(())
        } else {
            Err(ConfigError::MalformedConfig)
        }
    }

    /// delete a key in a section of the config
    #[allow(dead_code)]
    pub fn delete_key(&mut self, section: &str, key: &str) -> Result<(), ConfigError> {
        if let Value::Table(config) = &mut self.config {
            if let Some(Value::Table(section_data)) = config.get_mut(section) {
                section_data.remove(key);
                Ok(())
            } else {
                Err(ConfigError::MissingSection(section.to_owned()))
            }
        } else {
            Err(ConfigError::MalformedConfig)
        }
    }

    /// Merge the loaded user config with a default configuration file.
    ///
    /// This is useful for automatic config updating when you update your default frontend config,
    /// as the new keys will be merged onto the user's existing config, without overwriting the
    /// keys that are already present.
    ///
    /// # Arguments
    ///
    /// * `default` - The default config that comes with the base installation of the frontend
    pub fn merge_with_default(&mut self, default: &str) -> Result<(), toml::de::Error> {
        fn merge(user_config: &mut Value, default_config: &Value) {
            if let (Value::Table(user_table), Value::Table(system_table)) =
                (user_config, default_config)
            {
                // Update keys in user table with keys from system table
                for (key, system_value) in system_table {
                    if !user_table.contains_key(key) {
                        // Key does not exist in user table, add it with system value
                        user_table.insert(key.clone(), system_value.clone());
                    } else {
                        // Key exists in both user and system tables, recursively update values
                        if let Some(user_value) = user_table.get_mut(key) {
                            merge(user_value, system_value);
                        }
                    }
                }
            }
        }

        let default_config: Value = toml::from_str(default)?;
        merge(&mut self.config, &default_config);
        Ok(())
    }
}

#[cfg(any(unix, windows))]
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
