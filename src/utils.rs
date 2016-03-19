use std::process::{self, Command, Stdio};
use std::str;

pub fn run(command: &mut Command) {
    let status = command.status().unwrap();

    if !status.success() {
        process::exit(status.code().unwrap());
    }
}

pub fn run_output(command: &mut Command) -> String {
    let output = command.stderr(Stdio::inherit()).output().unwrap();
    if !output.status.success() {
        print!("{}", str::from_utf8(&output.stdout).unwrap());
        process::exit(output.status.code().unwrap());
    }

    String::from_utf8(output.stdout).unwrap()
}

pub fn run_filter(filter: &str, command: &mut Command) {
    let replacement = String::from_utf8(vec![b'X'; filter.len()]).unwrap();

    print!("{}", run_output(command).replace(filter, &replacement));
}
