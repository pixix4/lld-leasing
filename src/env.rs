lazy_static::lazy_static! {
    pub static ref HTTP_PORT: u16 = std::env::var("HTTP_PORT")
        .unwrap_or_else(|_| "".to_string())
        .parse::<u16>()
        .unwrap_or(3030);
    pub static ref HTTP_REQUEST_URI: String = std::env::var("HTTP_REQUEST_URI")
        .unwrap_or_else(|_| "http://localhost:3030/request".to_owned());

    pub static ref TCP_PORT: u16 = std::env::var("TCP_PORT")
        .unwrap_or_else(|_| "".to_string())
        .parse::<u16>()
        .unwrap_or(3040);
    pub static ref TCP_REQUEST_URI: String = std::env::var("TCP_REQUEST_URI")
        .unwrap_or_else(|_| "127.0.0.1:3040".to_owned());
}
