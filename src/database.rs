use core::fmt;
use std::collections::HashMap;

use std::fmt::Write;

use crate::{cache::CacheMap, LldResult};

#[derive(Debug)]
pub enum DatabaseTask {
    Insert {
        application_id: String,
        instance_id: String,
        validity: u64,
    },
    Update {
        application_id: String,
        instance_id: String,
        validity: u64,
    },
}

impl DatabaseTask {
    pub fn get_validity(&self) -> u64 {
        match self {
            DatabaseTask::Insert {
                application_id: _,
                instance_id: _,
                validity,
            } => *validity,
            DatabaseTask::Update {
                application_id: _,
                instance_id: _,
                validity,
            } => *validity,
        }
    }
}

#[cfg(not(feature = "dqlite"))]
mod database_connection {

    use crate::sqlite::Connection as SqliteConnection;
    use crate::{env, LldResult};

    use super::DatabaseValue;

    pub struct Connection {
        connection: SqliteConnection,
    }

    impl Connection {
        pub fn open() -> LldResult<Self> {
            let connection = SqliteConnection::open(env::DATABASE_URI.as_str())?;
            Ok(Self { connection })
        }

        pub fn execute(&self, statement: &str) -> LldResult<()> {
            self.connection.execute(statement)?;
            Ok(())
        }

        pub fn iterate<T: AsRef<str>, F>(&self, statement: T, mut callback: F) -> LldResult<()>
        where
            F: FnMut(&[(String, DatabaseValue)]) -> bool,
        {
            self.connection.iterate(statement, |fields| {
                let vec: Vec<(String, DatabaseValue)> = fields
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.to_owned().to_owned(),
                            match value {
                                Some(value) => DatabaseValue::Text(value.to_owned().to_owned()),
                                None => DatabaseValue::Null(),
                            },
                        )
                    })
                    .collect();

                callback(&vec)
            })?;
            Ok(())
        }
    }
}

#[cfg(feature = "dqlite")]
mod database_connection {

    use crate::dqlite::Connection as DqliteConnection;
    use crate::dqlite::DqliteValueWrapper;
    use crate::LldResult;

    use super::DatabaseValue;

    pub struct Connection {
        connection: DqliteConnection,
    }

    impl Connection {
        pub fn open() -> LldResult<Self> {
            let connection = DqliteConnection::open("leasings")?;
            Ok(Self { connection })
        }

        pub fn execute(&self, statement: &str) -> LldResult<()> {
            self.connection.execute(statement)?;
            Ok(())
        }

        pub fn iterate<T: AsRef<str>, F>(&self, statement: T, mut callback: F) -> LldResult<()>
        where
            F: FnMut(&[(String, DatabaseValue)]) -> bool,
        {
            self.connection.iterate(statement, |fields| {
                let vec: Vec<(String, DatabaseValue)> = fields
                    .iter()
                    .map(|(key, value)| {
                        (
                            key.to_owned(),
                            match value {
                                DqliteValueWrapper::Integer(x) => DatabaseValue::Integer(*x),
                                DqliteValueWrapper::Float(x) => DatabaseValue::Float(*x),
                                DqliteValueWrapper::Null() => DatabaseValue::Null(),
                                DqliteValueWrapper::Text(x) => DatabaseValue::Text(x.clone()),
                                DqliteValueWrapper::Boolean(x) => DatabaseValue::Boolean(*x),
                                DqliteValueWrapper::Unknown() => DatabaseValue::Unknown(),
                            },
                        )
                    })
                    .collect();

                return callback(&vec);
            })?;
            Ok(())
        }
    }
}

pub enum DatabaseValue {
    Integer(i64),
    Float(f64),
    Null(),
    Text(String),
    Boolean(bool),
    Unknown(),
}

impl DatabaseValue {
    pub fn to_u64(&self) -> u64 {
        match self {
            Self::Integer(x) => *x as u64,
            Self::Float(x) => *x as u64,
            Self::Null() => 0,
            Self::Text(x) => x.parse::<u64>().unwrap_or(0),
            Self::Boolean(x) => *x as u64,
            Self::Unknown() => 0,
        }
    }
}

impl fmt::Display for DatabaseValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Integer(x) => write!(f, "{}", x),
            Self::Float(x) => write!(f, "{}", x),
            Self::Null() => write!(f, ""),
            Self::Text(x) => write!(f, "{}", x),
            Self::Boolean(x) => write!(f, "{}", x),
            Self::Unknown() => write!(f, ""),
        }
    }
}

pub struct Database {
    connection: database_connection::Connection,
}

impl fmt::Debug for Database {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Database").finish()
    }
}

impl Database {
    pub fn open() -> LldResult<Self> {
        let connection = database_connection::Connection::open()?;
        Ok(Self { connection })
    }

    pub fn init(&self) -> LldResult<()> {
        self.connection
            .execute(r#"DROP TABLE IF EXISTS leasings;"#)?;
        self.connection.execute(
            r#"CREATE TABLE leasings (
                application_id TEXT NOT NULL PRIMARY KEY,
                instance_id TEXT NOT NULL,
                validity INTEGER NOT NULL
);"#,
        )?;

        self.build_cache()?;

        Ok(())
    }

    pub fn build_cache(&self) -> LldResult<CacheMap> {
        let mut cache: CacheMap = HashMap::new();

        self.connection.iterate(
            "SELECT application_id, instance_id, validity FROM leasings;",
            |pairs| {
                let application_id = pairs[0].1.to_string();
                let instance_id = pairs[1].1.to_string();
                let validity = pairs[2].1.to_u64();
                cache.insert(application_id, (instance_id, validity));
                true
            },
        )?;

        Ok(cache)
    }

    pub fn query_leasing(&self, application_id: &str) -> LldResult<Option<(String, u64)>> {
        let mut result: Option<(String, u64)> = None;
        self.connection.iterate(
            format!(
                "SELECT instance_id, validity FROM leasings WHERE application_id='{}';",
                application_id
            ),
            |pairs| {
                let instance_id = pairs[0].1.to_string();
                let validity = pairs[1].1.to_u64();
                result = Some((instance_id, validity));
                true
            },
        )?;

        Ok(result)
    }

    fn get_update_leasing_sql(application_id: &str, instance_id: &str, validity: u64) -> String {
        format!(
            "UPDATE leasings SET validity = {}, instance_id = '{}' WHERE application_id = '{}';",
            validity, instance_id, application_id
        )
    }

    pub fn update_leasing(
        &self,
        application_id: &str,
        instance_id: &str,
        validity: u64,
    ) -> LldResult<bool> {
        self.connection.execute(
            Database::get_update_leasing_sql(application_id, instance_id, validity).as_str(),
        )?;

        Ok(true)
    }

    fn get_insert_leasing_sql(application_id: &str, instance_id: &str, validity: u64) -> String {
        format!(
            "INSERT INTO leasings (application_id, instance_id, validity) VALUES ('{}', '{}', {});",
            application_id, instance_id, validity
        )
    }

    pub fn insert_leasing(
        &self,
        application_id: &str,
        instance_id: &str,
        validity: u64,
    ) -> LldResult<bool> {
        self.connection.execute(
            Database::get_insert_leasing_sql(application_id, instance_id, validity).as_str(),
        )?;

        Ok(true)
    }

    pub fn execute_tasks(&self, tasks: &[DatabaseTask]) -> LldResult<bool> {
        let mut transaction = String::new();

        for task in tasks {
            let statement = match task {
                DatabaseTask::Insert {
                    application_id,
                    instance_id,
                    validity,
                } => Database::get_insert_leasing_sql(application_id, instance_id, *validity),
                DatabaseTask::Update {
                    application_id,
                    instance_id,
                    validity,
                } => Database::get_update_leasing_sql(application_id, instance_id, *validity),
            };

            writeln!(&mut transaction, "{}", statement)?;
        }

        self.connection.execute(transaction.as_str())?;
        Ok(true)
    }
}
