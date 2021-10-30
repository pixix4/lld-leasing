use sqlite::{Connection, State};

use crate::{env, utils::get_current_time, LldResult};

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
                instance_id TEXT NOT NULL,
                application_id TEXT NOT NULL PRIMARY KEY,
                validity INTEGER NOT NULL
);"#,
        )?;
        Ok(())
    }

    pub fn request_leasing(
        &self,
        instance_id: &str,
        application_id: &str,
        duration: u64,
    ) -> LldResult<Option<u64>> {
        let now = get_current_time();

        let mut statement = self
            .connection
            .prepare("SELECT instance_id, validity FROM leasings WHERE application_id = ?")?;

        statement.bind(1, application_id)?;

        let mut found: Option<(String, i64)> = None;
        while let State::Row = statement.next()? {
            found = Some((statement.read::<String>(0)?, statement.read::<i64>(1)?));
        }

        Ok(match found {
            Some((leased_instance_id, validity)) => {
                let validity = validity as u64;
                if validity > now && leased_instance_id != instance_id {
                    None
                } else {
                    let mut statement = self
                        .connection
                        .prepare("UPDATE leasings SET validity = ?, instance_id = ? WHERE application_id = ?")?;

                    statement.bind(1, (now + duration) as i64)?;
                    statement.bind(2, instance_id)?;
                    statement.bind(3, application_id)?;

                    statement.next()?;

                    Some(now + duration)
                }
            }
            None => {
                let mut statement = self.connection.prepare(
                    "INSERT INTO leasings (instance_id, application_id, validity) VALUES (?, ?, ?)",
                )?;

                statement.bind(1, instance_id)?;
                statement.bind(2, application_id)?;
                statement.bind(3, (now + duration) as i64)?;

                statement.next()?;
                Some(now + duration)
            }
        })
    }
}
