extern crate docopt;
extern crate regex;
extern crate rustc_serialize;

pub mod cargo;
pub mod coverage;
pub mod doc_upload;
pub mod manifest;
pub mod utils;

pub use manifest::{Manifest, Target};
