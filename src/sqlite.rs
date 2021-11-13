//use std::{thread::sleep, time::Duration};

use sqlite::Connection;

use crate::LldResult;

pub struct Sqlite {
    connection: Connection,
}

impl Sqlite {
    pub fn open(path: &str) -> LldResult<Self> {
        //sleep(Duration::from_millis(10));
        let connection = sqlite::open(path)?;
        Ok(Self { connection })
    }

    pub fn execute(&self, statement: &str) -> LldResult<()> {
        //sleep(Duration::from_millis(10));
        self.connection.execute(statement)?;
        Ok(())
    }

    pub fn iterate<T: AsRef<str>, F>(&self, statement: T, callback: F) -> LldResult<()>
    where
        F: FnMut(&[(&str, Option<&str>)]) -> bool,
    {
        //sleep(Duration::from_millis(10));
        self.connection.iterate(statement, callback)?;
        Ok(())
    }
}
