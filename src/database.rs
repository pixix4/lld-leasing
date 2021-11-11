use std::collections::HashMap;

use sqlite::Connection;
use std::fmt::Write;

use crate::{cache::CacheMap, env, LldResult};

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

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn open() -> LldResult<Self> {
        let connection = sqlite::open(env::DATABASE_URI.as_str())?;
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
                let application_id = pairs[0].1.unwrap_or("");
                let instance_id = pairs[1].1.unwrap_or("");
                let validity = pairs[2].1.unwrap_or("").parse::<u64>().unwrap_or(0);
                cache.insert(
                    application_id.to_owned(),
                    (instance_id.to_owned(), validity),
                );
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
                let instance_id = pairs[0].1.unwrap_or("");
                let validity = pairs[1].1.unwrap_or("").parse::<u64>().unwrap_or(0);
                result = Some((instance_id.to_owned(), validity));
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
        self.connection.execute(Database::get_update_leasing_sql(
            application_id,
            instance_id,
            validity,
        ))?;

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
        self.connection.execute(Database::get_insert_leasing_sql(
            application_id,
            instance_id,
            validity,
        ))?;

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

        self.connection.execute(transaction)?;
        Ok(true)
    }
}
