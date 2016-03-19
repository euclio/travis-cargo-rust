extern crate docopt;
extern crate rustc_serialize;
extern crate travis_cargo;

use std::env;
use std::process::Command;

use docopt::Docopt;

use travis_cargo::{Manifest, cargo, doc_upload, coverage, utils};

const USAGE: &'static str = r"
Manages interactions between Travis and Cargo and common tooling tasks.

Usage:
    travis-cargo [-h] [-q] [--only VERSION] [--skip VERSION] <command> [<args>...]

Options:
    --help -h       show this screen
    --quiet -q      don't pass --verbose to cargo subcommands
    --only VERSION  only run the given command if the specified version matches `TRAVIS RUST VERSION`
    --skip VERSION  only run the given command if the specified version does not match `TRAVIS RUST VERSION`

Subcommands:
  travis-cargo supports all cargo subcommands, and selected others (listed below). Cargo
  subcommands have `--verbose` added to their invocation by default, and, when running with a
  nightly compiler, `--features unstable` (or `--features $TRAVIS_CARGO_NIGHTLY_FEATURE` if that
  environment variable is defined) if `--features` is a valid argument.

    coverage        record code coverage
    coveralls       record and upload code coverage to coveralls.io
    doc-upload      upload documentation to GitHub pages
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_command: String,
    arg_args: Vec<String>,
    flag_quiet: bool,
    flag_only: Option<String>,
    flag_skip: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|dopt| dopt.options_first(true).decode())
                         .unwrap_or_else(|e| e.exit());

    let version = env::var("TRAVIS_RUST_VERSION")
                      .ok()
                      .unwrap_or_else(|| {
                          // fill in the version based on the compiler's version output.
                          let output = utils::run_output(Command::new("rustc").arg("-V"));

                          let phrases = ["nightly", "dev", "beta"];
                          phrases.iter()
                                 .map(|&phrase| {
                                     match phrase {
                                         "dev" => "nightly",
                                         _ => phrase,
                                     }
                                 })
                                 .find(|&phrase| output.contains(phrase))
                                 .unwrap_or_default()
                                 .to_owned()
                      });

    if args.flag_only.map(|only| only != version).unwrap_or_default() {
        return;
    }

    if args.flag_skip.map(|skip| skip == version).unwrap_or_default() {
        return;
    }

    let manifest = Manifest::new(env::current_dir().unwrap(), &version);
    match &args.arg_command[..] {
        "doc-upload" => doc_upload::doc_upload(manifest),
        "coverage" => coverage::coverage(&version),
        "coveralls" => coverage::coveralls(&version),
        ref command @ _ => {
            if ["build", "bench", "test", "doc", "run", "rustc", "rustdoc"].contains(command) {
                cargo::cargo_feature(&version, args.flag_quiet, &args.arg_command, &args.arg_args);
            } else {
                cargo::cargo_no_feature(&version,
                                        args.flag_quiet,
                                        &args.arg_command,
                                        &args.arg_args);
            }
        }
    }
}
