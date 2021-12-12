fn main() {
    println!("cargo:rustc-link-search=/usr/local/lib");
    println!("cargo:rustc-link-arg=-lsqlite3");

    if cfg!(feature = "dqlite") {
        println!("cargo:rustc-link-arg=-ldqlite");
        println!("cargo:rustc-link-arg=-lraft");
        println!("cargo:rustc-link-arg=-ldqlitec");
    }
}
