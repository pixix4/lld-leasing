mod ffi;

use std::ffi::CString;

use crate::{
    dqlite::ffi::{Dqlite, DqliteRows},
    LldError, LldResult,
};

use libc::c_int;

macro_rules! raise(
    ($message:expr) => (
        return Err(LldError::DatabaseError {
            code: None,
            message: Some($message.to_string()),
        })
    );
);

macro_rules! c_str_to_str(
    ($string:expr) => (::std::str::from_utf8(::std::ffi::CStr::from_ptr($string).to_bytes()));
);

macro_rules! str_to_cstr(
    ($string:expr) => (
        match ::std::ffi::CString::new($string) {
            Ok(string) => string,
            _ => raise!("failed to process a string"),
        }
    );
);

/// A database connection.
pub struct Connection {}

unsafe impl Send for Connection {}

impl Connection {
    pub fn open(database_name: &str) -> LldResult<Connection> {
        let ipc = std::fs::read_to_string("ips.csv")
            .unwrap()
            .split("\n")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_owned())
            .map(|s| CString::new(s).unwrap())
            .collect::<Vec<CString>>();

        unsafe {
            ffi::set_n_clients(3);
            ffi::init_ips(str_to_cstr!("ips.csv").as_ptr());
            println!("clients: {}", ffi::get_n_servers());

            for i in 0..ipc.len() {
                // let ip = ::std::ffi::CStr::from_ptr(ffi::get_ip(i as i32));
                let ip = &ipc[i];
                println!("Connecting to socket {:?} {:?}", ip, ip.as_ptr());

                let mut fd = 0;
                let res = ffi::connect_socket(&mut fd as *mut c_int, ip.as_ptr());
                println!("Connected to socket {:?}: {}", ip, res);
                if res != 0 {
                    raise!("Cannot connect socket!");
                }

                println!("Init client {:?}", ip);
                let client = &mut ffi::clients[i];
                let res = ffi::clientInit(client as *mut Dqlite, fd as c_int);
                println!("Init client {:?} finished: {}", ip, res);

                println!("Handshake to client {:?}", ip);
                let res = ffi::clientSendHandshake(client as *mut Dqlite);
                println!("Handshake to client {:?} finished {}", ip, res);
                if res != 0 {
                    raise!("Handshake failed!");
                }
            }

            let mut i: usize = 0;
            for client in &mut ffi::clients {
                if i == 0 {
                    i += 1;
                    continue;
                }

                // let ip = ::std::ffi::CStr::from_ptr(ffi::get_ip(i as i32));
                let ip = &ipc[i];
                println!("Adding server {:?} {:?}", ip, ip.as_ptr());
                let res = ffi::addServer(client as *mut Dqlite, i as u32, ip.as_ptr());
                println!("Added server {:?}: {}", ip, res);

                i += 1;
            }

            println!("Send open {}", database_name);
            let res = ffi::send_open(str_to_cstr!(database_name).as_ptr());

            println!("Send open {} finished: {}", database_name, res);
            if res != 0 {
                raise!("Cannot open database!");
            }
        }

        Ok(Connection {})
    }

    /// Execute a statement without processing the resulting rows if any.
    #[inline]
    pub fn execute<T: AsRef<str>>(&self, statement: T) -> LldResult<()> {
        unsafe {
            let res = ffi::exec(str_to_cstr!(statement.as_ref()).as_ptr());
            if res != 0 {
                raise!("Cannot exec statement!");
            }
        }
        Ok(())
    }

    /// Execute a statement and process the resulting rows as plain text.
    ///
    /// The callback is triggered for each row. If the callback returns `false`,
    /// no more rows will be processed. For large queries and non-string data
    /// types, prepared statement are highly preferable; see `prepare`.
    #[inline]
    pub fn iterate<T: AsRef<str>, F>(&self, statement: T, mut callback: F) -> LldResult<()>
    where
        F: FnMut(&[(String, DqliteValueWrapper)]) -> bool,
    {
        unsafe {
            let mut rows = std::mem::MaybeUninit::<DqliteRows>::zeroed().assume_init();

            println!("1");
            let res = ffi::raw_query(
                &mut rows as *mut DqliteRows,
                str_to_cstr!(statement.as_ref()).as_ptr(),
            );
            if res != 0 {
                raise!("Cannot exec statement!");
            }
            println!("2 {}", rows.column_count);

            let args = (0..rows.column_count)
                .map(|i| {
                    i.to_string()
                    // println!("2.1 {}", i);
                    // let name = rows.column_names.offset(i as isize);

                    // println!("2.2 {:?}", name);
                    // println!("2.3 {:?}", c_str_to_str!(*name));
                    // return c_str_to_str!(*name)
                    //     .map(|s| s.to_owned())
                    //     .unwrap_or_else(|_| i.to_string());
                })
                .collect::<Vec<String>>();

            if rows.next.is_null() {
                return Ok(());
            }

            println!("3");

            let column_count = rows.column_count;
            let mut this_row = rows.next;

            println!("4");

            while !this_row.is_null() {
                let mut result =
                    Vec::<(String, DqliteValueWrapper)>::with_capacity(column_count as usize);

                println!("5");
                for i in 0..column_count {
                    let union_type = (*(*this_row).values.offset(i as isize)).union_type;

                    println!("6: {}: {}", i, union_type);
                    let value = match union_type {
                        1 => DqliteValueWrapper::Integer(
                            (*(*this_row).values.offset(i as isize)).union_value.integer,
                        ),
                        2 => DqliteValueWrapper::Float(
                            (*(*this_row).values.offset(i as isize)).union_value.float_,
                        ),
                        5 => DqliteValueWrapper::Null(),
                        11 => DqliteValueWrapper::Boolean(
                            (*(*this_row).values.offset(i as isize)).union_value.boolean != 0,
                        ),
                        3 => DqliteValueWrapper::Text(
                            c_str_to_str!(
                                (*(*this_row).values.offset(i as isize)).union_value.text
                            )
                            .unwrap()
                            .to_owned(),
                        ),
                        _ => DqliteValueWrapper::Unknown(),
                    };

                    result.push((args[i as usize].clone(), value));
                }

                println!("8");

                if !callback(&result) {
                    break;
                }

                this_row = (*this_row).next;
            }
        }

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {}
}

#[derive(Debug, Clone)]
pub enum DqliteValueWrapper {
    Integer(i64),
    Float(f64),
    Null(),
    Text(String),
    Boolean(bool),
    Unknown(),
}
