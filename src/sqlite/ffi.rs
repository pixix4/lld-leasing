use libc::{c_char, c_int, c_void};

pub const SQLITE_OK: c_int = 0;

pub enum Sqlite3 {}

pub type Sqlite3ExecCallback =
    extern "C" fn(*mut c_void, c_int, *mut *mut c_char, *mut *mut c_char) -> c_int;

extern "C" {

    pub fn sqlite3_close(p: *mut Sqlite3) -> c_int;

    pub fn sqlite3_open_v2(
        filename: *const c_char,
        pp_db: *mut *mut Sqlite3,
        flags: c_int,
        z_vfs: *const c_char,
    ) -> c_int;

    pub fn sqlite3_exec(
        sqlite3: *mut Sqlite3,
        sql: *const c_char,
        callback: Option<Sqlite3ExecCallback>,
        arg: *mut c_void,
        errmsg: *mut *mut c_char,
    ) -> c_int;
}
