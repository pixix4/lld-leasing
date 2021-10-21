use std::time::{SystemTime, UNIX_EPOCH};

use sqlite::{Connection, State};

use crate::LldResult;

pub fn get_current_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

pub struct Database {
    connection: Connection,
}

impl Database {
    pub fn init() -> LldResult<Self> {
        let connection = sqlite::open(":memory:")?;

        connection.execute(
            r#"CREATE TABLE leasings (
    id TEXT NOT NULL PRIMARY KEY,
    validity INTEGER NOT NULL
);"#,
        )?;

        Ok(Self { connection })
    }

    pub fn request_leasing(&self, id: &str) -> LldResult<Option<i64>> {
        let now = get_current_time();

        println!("# Request leasing for {}", id);
        let mut statement = self
            .connection
            .prepare("SELECT id, validity FROM leasings WHERE id = ?")?;

        statement.bind(1, id)?;

        let mut found: Option<i64> = None;
        while let State::Row = statement.next()? {
            found = Some(statement.read::<i64>(1)?);
        }

        Ok(match found {
            Some(validity) => {
                if validity > now {
                    None
                } else {
                    let mut statement = self
                        .connection
                        .prepare("UPDATE leasings SET validity = ? WHERE id = ?")?;

                    statement.bind(1, now + 5000)?;
                    statement.bind(2, id)?;

                    statement.next()?;

                    Some(now + 5000)
                }
            }
            None => {
                let mut statement = self
                    .connection
                    .prepare("INSERT INTO leasings (id, validity) VALUES (?, ?)")?;

                statement.bind(1, id)?;
                statement.bind(2, now + 5000)?;

                statement.next()?;
                Some(now + 5000)
            }
        })
    }
}
