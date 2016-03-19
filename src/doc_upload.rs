use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

use docopt::Docopt;

use Manifest;
use utils;

const USAGE: &'static str = r"
usage: travis_cargo doc-upload [-h] [--branch BRANCH]

Use ghp-import to upload cargo-rendered docs to GitHub Pages, from the master
branch.

optional arguments:
  -h, --help       show this help message and exit
  --branch BRANCH  upload docs when on this branch, defaults to master";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_branch: Option<String>,
}

pub fn doc_upload(manifest: Manifest) {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|dopt| dopt.decode())
                         .unwrap_or_else(|e| e.exit());

    let branch = env::var("APPVEYOR_REPO_BRANCH")
                     .or(env::var("TRAVIS_BRANCH"))
                     .ok()
                     .expect("branch not found");
    let repo = env::var("APPVEYOR_REPO_NAME")
                   .or(env::var("TRAVIS_REPO_SLUG"))
                   .ok()
                   .expect("repo name not found");
    let pr = match env::var("APPVEYOR_PULL_REQUEST_NUMBER") {
        Ok(_) => true,
        Err(_) => env::var("TRAVIS_PULL_REQUEST").map(|pr| pr.parse().unwrap()).unwrap(),
    };

    let lib_name = manifest.lib_name().unwrap();
    if branch == args.flag_branch.unwrap_or("master".to_owned()) && !pr {
        // only load the token when we're sure we're uploading (travis
        // won't decrypt secret keys for PRs, so loading this with the
        // other vars causes problems with tests)
        let token = env::var("GH_TOKEN").unwrap();
        println!("uploading docs...");
        let mut file = File::create("target/doc/index.html").unwrap();
        writeln!(file,
                 "<meta http-equiv=refresh content=0;url={}/index.html>",
                 &lib_name)
            .unwrap();

        utils::run(Command::new("git").args(&["clone", "https://github.com/davisp/ghp-import"]));
        utils::run(Command::new("python")
                       .args(&["./ghp-import/ghp_import.py", "-n", "target/doc"]));
        let repo_url = format!("https://{}@github.com/{}.git", token, repo);
        utils::run_filter(&token,
                          Command::new("git").args(&["push", "-fq", &repo_url, "gh-pages"]));
    }
}
