lazy_static::lazy_static! {
    pub static ref REQUEST_URI: String = std::env::var("REQUEST_URI")
        .unwrap_or_else(|_| "http://localhost:3030/request".to_owned());

    pub static ref PORT: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "".to_string())
        .parse::<u16>()
        .unwrap_or(3030);
}
