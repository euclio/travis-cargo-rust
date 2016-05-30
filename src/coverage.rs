use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use docopt::Docopt;
use regex::Regex;

use cargo;
use utils;

const COVERAGE_USAGE: &'static str = r#"
Usage:
    travis-cargo coverage [options] [--] [<args>...]

Record coverage of `cargo test`, this runs all binaries that `cargo test` runs but not doc tests.
The results of all tests are merged into a single directory.

positional arguments:
    args                  arguments to pass to `cargo test`

optional arguments:
    -h, --help            show this help message and exit

    -m DIR, --merge-into DIR
                          the directory to put the final merged kcov result into (default `target/kcov`)

    --no-sudo             don't use `sudo` to install kcov's deps. Requires that
                          libcurl4-openssl-dev, libelf-dev and libdw-dev are installed (e.g., via
                          `addons: apt: packages:`)

    --verify              pass `--verify` to kcov, to avoid some crashes. See
                          <https://github.com/huonw/travis-cargo/issues/12>. This requires
                          installing the `binutils-dev` package.

    --exclude-pattern PATTERN
                          pass additional comma-separated exclusionary patterns to kcov. See
                          <https://github.com/SimonKagstrom/kcov#filtering-output> for how patterns
                          work. By default, the /.cargo pattern is ignored. Example:
                          `--exclude-pattern="test/,bench/"`

    --kcov-options OPTION ...
                         pass additional arguments to kcov, apart from `--verify` and
                         `--exclude-pattern`, when recording coverage. Specify multiple times for
                         multiple arguments. Example: --kcov-options="--debug=31"

    --no-link-dead-code
                        By default, travis_cargo passes `-C link-dead-code` to rustc during
                        compilation. This can lead to more accurate code coverage if your compiler
                        supports it. This flags allows users to opt out of linking dead code.
"#;

#[derive(Debug, RustcDecodable)]
struct CoverageArgs {
    flag_merge_into: Option<String>,
    flag_no_sudo: bool,
    flag_verify: bool,
    flag_no_link_dead_code: bool,
    arg_args: Vec<String>,
    flag_kcov_args: Vec<String>,
    flag_exclude_pattern: Option<String>,
}

const COVERALLS_USAGE: &'static str = r#"
Usage:
    travis-cargo coveralls [options] [--] [<args>...]

Record coverage of `cargo test` and upload to coveralls.io with kcov, this
runs all binaries that `cargo test` runs but not doc tests. Merged kcov
results can be accessed in `target/kcov`.

positional arguments:
    args                  arguments to pass to `cargo test`

optional arguments:
    -h, --help            show this help message and exit

    -m DIR, --merge-into DIR
                          the directory to put the final merged kcov result into (default `target/kcov`)

    --no-sudo             don't use `sudo` to install kcov's deps. Requires that
                          libcurl4-openssl-dev, libelf-dev and libdw-dev are installed (e.g., via
                          `addons: apt: packages:`)

    --verify              pass `--verify` to kcov, to avoid some crashes. See
                          <https://github.com/huonw/travis-cargo/issues/12>. This requires
                          installing the `binutils-dev` package.

    --exclude-pattern PATTERN
                          pass additional comma-separated exclusionary patterns to kcov. See
                          <https://github.com/SimonKagstrom/kcov#filtering-output> for how patterns
                          work. By default, the /.cargo pattern is ignored. Example:
                          `--exclude-pattern="test/,bench/"`

    --kcov-options OPTION ...
                         pass additional arguments to kcov, apart from `--verify` and
                         `--exclude-pattern`, when recording coverage. Specify multiple times for
                         multiple arguments. Example: --kcov-options="--debug=31"

    --no-link-dead-code
                        By default, travis_cargo passes `-C link-dead-code` to rustc during
                        compilation. This can lead to more accurate code coverage if your compiler
                        supports it. This flags allows users to opt out of linking dead code.
"#;

#[derive(Debug, RustcDecodable)]
struct CoverallsArgs {
    flag_merge_into: Option<String>,
    flag_no_sudo: bool,
    flag_verify: bool,
    flag_no_link_dead_code: bool,
    arg_args: Vec<String>,
    flag_kcov_args: Vec<String>,
    flag_exclude_pattern: Option<String>,
}

pub fn coverage(version: &str) {
    let args: CoverageArgs = Docopt::new(COVERAGE_USAGE)
                                 .and_then(|dopt| dopt.decode())
                                 .unwrap_or_else(|e| e.exit());

    let mut cargo_args = args.arg_args.iter().cloned().collect();
    cargo::add_features(&mut cargo_args, version);

    let kcov_merge_dir = args.flag_merge_into.unwrap_or("target/kcov".into());
    raw_coverage(!args.flag_no_sudo,
                 args.flag_verify,
                 !args.flag_no_link_dead_code,
                 &cargo_args,
                 "Merging coverage",
                 &[],
                 kcov_merge_dir,
                 args.flag_exclude_pattern,
                 &args.flag_kcov_args);
}

pub fn coveralls(version: &str) {
    let args: CoverallsArgs = Docopt::new(COVERALLS_USAGE)
                                  .and_then(|dopt| dopt.decode())
                                  .unwrap_or_else(|e| e.exit());

    let job_id = env::var("TRAVIS_JOB_ID").unwrap();

    let mut cargo_args = args.arg_args.iter().cloned().collect();
    cargo::add_features(&mut cargo_args, version);

    let kcov_merge_dir = args.flag_merge_into.unwrap_or("target/kcov".into());
    raw_coverage(!args.flag_no_sudo,
                 args.flag_verify,
                 !args.flag_no_link_dead_code,
                 &cargo_args,
                 "Uploading coverage",
                 &[format!("--coveralls-id={}", job_id)],
                 kcov_merge_dir,
                 args.flag_exclude_pattern,
                 &args.flag_kcov_args);
}

fn build_kcov(use_sudo: bool, verify: bool) -> PathBuf {
    let mut init = String::new();

    if use_sudo {
        init.push_str("sudo apt-get install libcurl4-openssl-dev libelf-dev libdw-dev cmake");

        if verify {
            init.push_str(" binutils-dev");
        }
    }

    init.push_str(r"
    wget https://github.com/SimonKagstrom/kcov/archive/master.zip
    unzip -o master.zip
    mkdir -p kcov-master/build
    ");

    for line in init.split("\n") {
        let line = line.trim();
        if !line.is_empty() {
            println!("Running: {:?}", line);
            let tokens: Vec<_> = line.split(" ").collect();
            utils::run(Command::new(tokens[0]).args(&tokens[1..]));
        }
    }

    let current = env::current_dir().unwrap();
    env::set_current_dir("kcov-master/build").unwrap();

    let build = r"
        cmake ..
        make
    ";
    for line in build.split("\n") {
        let line = line.trim();
        if !line.is_empty() {
            println!("Running: {:?}", line);
            let tokens: Vec<_> = line.split(" ").collect();
            utils::run(Command::new(tokens[0]).args(&tokens[1..]));
        }
    }

    env::set_current_dir(&current).unwrap();
    current.join("kcov-master/build/src/kcov")
}


fn raw_coverage<P>(use_sudo: bool,
                   verify: bool,
                   link_dead_code: bool,
                   test_args: &[String],
                   merge_message: &str,
                   kcov_merge_args: &[String],
                   kcov_merge_dir: P,
                   exclude_pattern: Option<String>,
                   extra_kcov_args: &[String])
    where P: AsRef<Path>
{
    let kcov = build_kcov(use_sudo, verify);

    let mut test_binaries = vec![];

    // Look through the output of `cargo test` to find the test binaries.
    // FIXME: the information cargo feeds us is inconsistent/inaccurate, so using hte output of
    // read-manifest is far too much trouble.
    let mut command = Command::new("cargo");
    command.arg("test").args(&test_args);

    if link_dead_code {
        let existing = env::var("RUSTFLAGS").unwrap_or(String::new());
        command.env("RUSTFLAGS", existing + " -C link-dead-code");
        // we''ll need to recompile everything with the new flag, so
        // let's start from the start
        utils::run(Command::new("cargo").arg("clean").arg("-v"));
    }

    let output = utils::run_output(&mut command);

    let running = Regex::new("(?m)^     Running target/debug/(.*)$").unwrap();
    for cap in running.captures_iter(&output) {
        test_binaries.push(cap.at(1).unwrap().to_owned());
    }

    // dynamic dependencies aren't found properly under kcov without a manual hint.
    let mut ld_library_path = env::var("LD_LIBRARY_PATH").unwrap_or(String::new());
    if !ld_library_path.is_empty() {
        ld_library_path.push_str(":")
    }
    ld_library_path.push_str("target/debug/deps");

    // Record coverage for each binary
    for binary in test_binaries.iter() {
        println!("Recording {}", binary);
        let mut kcov_args: Vec<String> = extra_kcov_args.iter()
                                                        .cloned()
                                                        .map(|arg| arg.to_owned())
                                                        .collect();

        if verify {
            kcov_args.push("--verify".to_owned());
        }

        let exclude_pattern_arg: String = {
            let exclude_pattern_arg = "--exclude-pattern=/.cargo";
            if let Some(ref additional_exclude) = exclude_pattern {
                format!("{},{}", exclude_pattern_arg, additional_exclude)
            } else {
                exclude_pattern_arg.to_owned()
            }
        };

        kcov_args.push(exclude_pattern_arg);
        kcov_args.push(format!("target/kcov-{}", binary));
        kcov_args.push(format!("target/debug/{}", binary));
        print!("Running: kcov ");
        for arg in kcov_args.iter() {
            print!("{} ", arg);
        }
        println!("");

        utils::run(Command::new(kcov.clone()).args(&kcov_args)
                       .env("LD_LIBRARY_PATH", &ld_library_path));
    }

    // Merge all the coverages and upload in one go
    println!("{}", merge_message);
    let mut kcov_args: Vec<String> = ["--merge".to_owned()]
                                         .iter()
                                         .chain(kcov_merge_args)
                                         .cloned()
                                         .collect();
    kcov_args.push(kcov_merge_dir.as_ref().to_str().unwrap().to_owned());
    for binary in test_binaries {
        kcov_args.push(format!("target/kcov-{}", binary));
    }

    utils::run(Command::new(kcov).args(&kcov_args));
}
