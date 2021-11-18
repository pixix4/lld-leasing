mod ffi;

use crate::{LldError, LldResult};

use libc::{c_char, c_int, c_void};
use std::marker::PhantomData;
use std::path::Path;

macro_rules! raise(
    ($message:expr) => (
        return Err(LldError::DatabaseError {
            code: None,
            message: Some($message.to_string()),
        })
    );
);

macro_rules! error(
    ($connection:expr, $code:expr) => (
        return Err(LldError::DatabaseError {
            code: Some($code as isize),
            message: None,
        })
    );
);

macro_rules! c_str_to_str(
    ($string:expr) => (::std::str::from_utf8(::std::ffi::CStr::from_ptr($string).to_bytes()));
);

macro_rules! path_to_cstr(
    ($path:expr) => (
        match $path.to_str() {
            Some(path) => {
                match ::std::ffi::CString::new(path) {
                    Ok(string) => string,
                    _ => raise!("failed to process a path"),
                }
            }
            _ => raise!("failed to process a path"),
        }
    );
);

macro_rules! str_to_cstr(
    ($string:expr) => (
        match ::std::ffi::CString::new($string) {
            Ok(string) => string,
            _ => raise!("failed to process a string"),
        }
    );
);

macro_rules! ok(
    ($connection:expr, $result:expr) => (
        match $result {
            ffi::SQLITE_OK => {}
            code => error!($connection, code),
        }
    );
    ($result:expr) => (
        match $result {
            ffi::SQLITE_OK => {}
            code => return Err(::Error {
                code: Some(code as isize),
                message: None,
            }),
        }
    );
);

/// A database connection.
pub struct Connection {
    raw: *mut ffi::Sqlite3,
    phantom: PhantomData<ffi::Sqlite3>,
}

unsafe impl Send for Connection {}

impl Connection {
    /// Open a read-write connection to a new or existing database.
    pub fn open<T: AsRef<Path>>(path: T) -> LldResult<Connection> {
        let mut raw = std::ptr::null_mut();
        unsafe {
            let code = ffi::sqlite3_open_v2(
                path_to_cstr!(path.as_ref()).as_ptr(),
                &mut raw,
                0x00000006,
                std::ptr::null(),
            );
            match code {
                ffi::SQLITE_OK => {}
                code => {
                    ffi::sqlite3_close(raw);
                    return Err(LldError::DatabaseError {
                        code: Some(code as isize),
                        message: None,
                    });
                }
            }
        }
        Ok(Connection {
            raw,
            phantom: PhantomData,
        })
    }

    /// Execute a statement without processing the resulting rows if any.
    #[inline]
    pub fn execute<T: AsRef<str>>(&self, statement: T) -> LldResult<()> {
        unsafe {
            ok!(
                self.raw,
                ffi::sqlite3_exec(
                    self.raw,
                    str_to_cstr!(statement.as_ref()).as_ptr(),
                    None,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            );
        }
        Ok(())
    }

    /// Execute a statement and process the resulting rows as plain text.
    ///
    /// The callback is triggered for each row. If the callback returns `false`,
    /// no more rows will be processed. For large queries and non-string data
    /// types, prepared statement are highly preferable; see `prepare`.
    #[inline]
    pub fn iterate<T: AsRef<str>, F>(&self, statement: T, callback: F) -> LldResult<()>
    where
        F: FnMut(&[(&str, Option<&str>)]) -> bool,
    {
        unsafe {
            let callback = Box::new(callback);
            ok!(
                self.raw,
                ffi::sqlite3_exec(
                    self.raw,
                    str_to_cstr!(statement.as_ref()).as_ptr(),
                    Some(process_callback::<F>),
                    &*callback as *const F as *mut F as *mut _,
                    std::ptr::null_mut(),
                )
            );
        }
        Ok(())
    }
}

impl Drop for Connection {
    #[inline]
    #[allow(unused_must_use)]
    fn drop(&mut self) {
        unsafe { ffi::sqlite3_close(self.raw) };
    }
}

extern "C" fn process_callback<F>(
    callback: *mut c_void,
    count: c_int,
    values: *mut *mut c_char,
    columns: *mut *mut c_char,
) -> c_int
where
    F: FnMut(&[(&str, Option<&str>)]) -> bool,
{
    unsafe {
        let mut pairs = Vec::with_capacity(count as usize);
        for i in 0..(count as isize) {
            let column = {
                let pointer = *columns.offset(i);
                debug_assert!(!pointer.is_null());
                c_str_to_str!(pointer).unwrap()
            };
            let value = {
                let pointer = *values.offset(i);
                if pointer.is_null() {
                    None
                } else {
                    Some(c_str_to_str!(pointer).unwrap())
                }
            };
            pairs.push((column, value));
        }
        if (*(callback as *mut F))(&pairs) {
            0
        } else {
            1
        }
    }
}
