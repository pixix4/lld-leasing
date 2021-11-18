use std::panic;

extern crate cc;
extern crate pkg_config;

fn main() {
    if pkg_config::find_library("sqlite3").is_err() {
        panic!("sqlite3 could not be found!")
    }

    if cfg!(feature = "dqlite") {
        println!("cargo:rustc-link-arg=-ldqlite");
        println!("cargo:rustc-link-arg=-lraft");
        println!("cargo:rustc-link-arg=-lsqlite3");
        println!("cargo:rustc-link-arg=-ldqlitec");

        if pkg_config::find_library("dqlite").is_err() {
            panic!("dqlite could not be found!")
        }
    }
}
