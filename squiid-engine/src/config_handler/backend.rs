#[cfg(any(unix, windows))]
pub mod fs {
    use std::path::PathBuf;

    use directories::{BaseDirs, ProjectDirs};

    use crate::config_handler::ConfigBackend;

    #[derive(Debug)]
    pub struct FsBackend {
        path: PathBuf,
    }

    impl FsBackend {
        pub fn new() -> Self {
            let path = Self::determine_config_path();
            Self { path }
        }

        // TODO: document this somewhere
        /// Determine where the config file should be placed
        ///
        /// Linux: `~/.config/squiid/`
        ///
        /// MacOS: `/Users/<NAME>/Library/Application Support/org.ImaginaryInfinity.Squiid/`
        ///
        /// Windows: `C:\Users\<NAME>\AppData\Roaming\ImaginaryInfinity\Squiid\config`
        ///
        /// Anything else: See Linux
        pub fn determine_config_path() -> PathBuf {
            // try to determine correct config path
            if let Some(proj_dirs) = ProjectDirs::from("net", "ImaginaryInfinity", "Squiid") {
                return proj_dirs.config_dir().join("config.toml");
            }

            // couldn't determine config path, default to home directory .config folder
            let home_dir = BaseDirs::new()
                .map(|d| d.home_dir().to_owned())
                .unwrap_or_default();
            home_dir.join(".config").join("squiid").join("config.toml")
        }
    }

    impl ConfigBackend for FsBackend {
        fn load(&self) -> Option<String> {
            std::fs::read_to_string(&self.path).ok()
        }

        fn save(&self, content: &str) -> Result<(), String> {
            if let Some(parent) = self.path.parent() && !parent.is_dir() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Err(e.to_string());
                }
            }

            match std::fs::write(&self.path, content) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            }
        }

        fn config_directory(&self) -> Option<PathBuf> {
            Some(self.path.clone())
        }
    }
}

pub mod noop {
    use crate::config_handler::ConfigBackend;

    #[derive(Debug)]
    pub struct NoopBackend {}

    impl NoopBackend {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl ConfigBackend for NoopBackend {
        fn load(&self) -> Option<String> {
            Some(String::new())
        }

        fn save(&self, _content: &str) -> Result<(), String> { Ok(()) }

        fn config_directory(&self) -> Option<std::path::PathBuf> {
            None
        }
    }
}
