use std::path::Path;
use std::process::Command;
use std::str;

use rustc_serialize::json::Json;

#[derive(Debug, Clone)]
pub struct Target(Json);

impl Target {
    pub fn binary_name(&self) -> Option<String> {
        self.0
            .find("name")
            .and_then(Json::as_string)
            .map(|name| name.replace("-", "_"))
            .and_then(|name| {
                self.0
                    .find_path(&["metadata", "extra_filename"])
                    .and_then(Json::as_string)
                    .map(|file_name| name + file_name)
            })
    }
}

#[derive(Debug)]
pub struct Manifest(Json);

impl Manifest {
    pub fn new<P>(dir: P) -> Self
        where P: AsRef<Path>
    {
        // the --manifest-path behaviour changed in https://github.com/rust-lang/cargo/pull/1955,
        // so we need to be careful to handle both
        let path_file = dir.as_ref().join("Cargo.toml");
        let path_dir = dir;

        let stdout = {
            let output = Command::new("cargo")
                             .args(&["read-manifest",
                                     "--manifest-path",
                                     path_file.to_str().unwrap()])
                             .output()
                             .unwrap();

            if output.status.success() {
                output.stdout
            } else {
                Command::new("cargo")
                    .args(&["read-manifest",
                            "--manifest-path",
                            path_dir.as_ref().to_str().unwrap()])
                    .output()
                    .unwrap()
                    .stdout
            }
        };

        Manifest(Json::from_str(&str::from_utf8(&stdout).unwrap()).unwrap())
    }

    pub fn targets(&self) -> Option<Vec<Target>> {
        let target_json = self.0
                              .find("targets")
                              .and_then(Json::as_array)
                              .unwrap();

        Some(target_json.iter()
                        .map(|target_json| Target(target_json.to_owned()))
                        .collect())
    }

    pub fn lib_name(&self) -> Option<String> {
        for target in self.targets().unwrap() {
            if target.0
                     .find("kind")
                     .unwrap()
                     .as_array()
                     .unwrap()
                     .contains(&Json::String("lib".into())) {
                return Some(target.0.find("name").unwrap().as_string().unwrap().replace("-", "_"));
            }
        }
        None
    }
}
