use std::env;
use std::process::Command;

use utils;

pub const SUBCOMMAND_WANTS_FEATURE: &'static [&'static str] = &[
    "build",
    "bench",
    "test",
    "doc",
    "install",
    "run",
    "rustc",
    "rustdoc",
    ];

pub fn run(version: &str, quiet: bool, command: &str, args: &[String]) {
    let mut cargo_args: Vec<String> = args.iter().cloned().collect();

    let feature = SUBCOMMAND_WANTS_FEATURE.contains(&command);

    if command == "bench" && version != "nightly" {
        println!("skipping `cargo bench` on non-nightly version");
        return;
    }

    if feature {
        add_features(&mut cargo_args, version);
    }

    if !quiet && !cargo_args.contains(&"--verbose".to_owned()) && !args.contains(&"-v".to_owned()) {
        cargo_args.push("--verbose".into());
    }

    utils::run(Command::new("cargo").arg(command).args(&cargo_args));
}

pub fn add_features(cargo_args: &mut Vec<String>, version: &str) {
    let nightly_feature = env::var("TRAVIS_CARGO_NIGHTLY_FEATURE").unwrap_or("unstable".to_owned());

    if version == "nightly" && nightly_feature != "" {
        // Only touch feature arguments when we are actually going to add something non-trivial,
        // avoids problems like that in issue #14 (can't use -p ... on nightly even with an empty
        // nightly feature).
        let mut added_feature = false;
        for i in 0..cargo_args.len() {
            if cargo_args[i] == "--features" {
                cargo_args[i + 1].push_str(&(String::from(" ") + &nightly_feature));
                added_feature = true;
            } else if cargo_args[i].starts_with("--features=") {
                cargo_args[i].push_str(&(String::from(" ") + &nightly_feature));
                added_feature = true;
            }
        }

        if !added_feature {
            cargo_args.push("--features".into());
            cargo_args.push(nightly_feature);
        }
    }
}
