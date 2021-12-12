use libc::{c_char, c_double, c_int, c_uint, c_void, size_t};
use std::fmt;

#[repr(C)]
#[derive(Debug)]
pub struct Dqlite {
    pub fd: c_int,
    pub db_id: c_uint,
    pub read: DqliteBuffer,
    pub write: DqliteBuffer,
}

#[repr(C)]
#[derive(Debug)]
pub struct DqliteBuffer {
    pub data: *mut c_void,
    pub page_size: c_uint,
    pub n_pages: c_uint,
    pub offset: size_t,
}

#[repr(C)]
#[derive(Debug)]
pub struct DqliteRow {
    pub values: *mut DqliteValue,
    pub next: *mut DqliteRow,
}

#[repr(C)]
#[derive(Debug)]
pub struct DqliteRows {
    pub column_count: c_uint,
    pub column_names: *mut *const c_char,
    pub next: *mut DqliteRow,
}

#[repr(C)]
pub union DqliteValueUnion {
    pub integer: i64,
    pub float_: c_double,
    pub blob: [u8; 12],
    pub null: u64,
    pub text: *mut c_char,
    pub iso8601: *mut c_char,
    pub unixtime: i64,
    pub boolean: u64,
}

impl fmt::Debug for DqliteValueUnion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DqliteValueUnion").finish()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct DqliteValue {
    pub union_type: c_int,
    pub union_value: DqliteValueUnion,
}

#[link(name = "sqlite3", kind = "static")]
extern "C" {}

#[link(name = "dqlite", kind = "static")]
extern "C" {

    pub fn clientInit(client: *mut Dqlite, fd: c_int) -> c_int;

    pub fn clientSendHandshake(client: *mut Dqlite) -> c_int;

    pub fn init_ips(ipsfile: *const c_char);

    pub fn get_n_servers() -> c_int;
}

#[link(name = "dqlitec", kind = "static")]
extern "C" {

    pub static mut clients: [Dqlite; 3];

    pub fn connect_socket(fd: *mut c_int, raw_str_address: *const c_char) -> c_int;

    pub fn addServer(client: *mut Dqlite, id: c_uint, address: *const c_char) -> c_int;

    pub fn send_open(database_name: *const c_char) -> c_int;

    pub fn exec(stmt: *const c_char) -> c_int;

    pub fn raw_query(rows: *mut DqliteRows, stmt: *const c_char) -> c_int;

    pub fn set_n_clients(new_n_clients: c_int);
}
