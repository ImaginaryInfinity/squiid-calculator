use colored::Colorize;
use std::{
    fmt,
    fs::File,
    io::Write,
    panic::{self, PanicHookInfo},
    path::PathBuf,
};

#[derive(Debug)]
struct EnvironmentDetails<'a> {
    version: &'a str,
    pkg_name: &'a str,
    crate_name: &'a str,
    arch: &'a str,
    os: &'a str,
}

impl fmt::Display for EnvironmentDetails<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Version: {}\nPackage: {}\nCrate: {}\nArchitecture: {}\nOS: {}",
            self.version, self.pkg_name, self.crate_name, self.arch, self.os,
        )
    }
}

pub fn crash_report(panic_info: &PanicHookInfo, config_path: Option<PathBuf>) {
    let backtrace = backtrace::Backtrace::new();

    // create environment struct
    let environment = EnvironmentDetails {
        version: env!("CARGO_PKG_VERSION"),
        pkg_name: env!("CARGO_PKG_NAME"),
        crate_name: env!("CARGO_CRATE_NAME"),
        arch: std::env::consts::ARCH,
        os: std::env::consts::OS,
    };

    // print crash report for user
    println!(
        "{}\n\n{}\n\n{}\n\n{}\n\n{}\n{:?}\n{}",
        "-".repeat(70).yellow(),
        "---------- Crash Report Information ----------".red(),
        environment,
        panic_info,
        "---------- Backtrace ----------".blue(),
        backtrace,
        "-".repeat(70).yellow(),
    );

    // determine the config directory to write the crash to
    if let Some(write_path) = config_path {
        let result = panic::catch_unwind(|| write_path);

        let config_directory = match result {
            Ok(value) => value.parent().map(|path| path.to_path_buf()),
            Err(_) => match std::env::current_dir() {
                Ok(value) => Some(value),
                Err(_) => None,
            },
        };

        if let Some(mut config_path_unwrapped) = config_directory {
            config_path_unwrapped.push("squiid_crash.txt");

            // remove the old crash if it exists
            let _ = std::fs::remove_file(&config_path_unwrapped);

            let file = File::create(&config_path_unwrapped);
            if let Ok(mut file_unwrapped) = file {
                // write crash file
                let crash_string = format!(
                    "Crash report generated at {}\n\n{}\n\n{}\n\n{:?}",
                    chrono::offset::Local::now(),
                    environment,
                    panic_info,
                    backtrace
                );
                let write_result = file_unwrapped.write_all(crash_string.as_bytes());
                if write_result.is_ok() {
                    println!(
                        "Crash report written to: {}",
                        config_path_unwrapped.to_string_lossy()
                    );
                }
            }
        }
    }

    println!("\n\nPlease report this issue at https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid/-/issues/new?issuable_template=Bug%20Report")
}
