//! Sqlite driver implementation
use super::DataSourceDriver;
use sqlite::{self, Connection, State};

pub struct SqliteDataSource {
    conn: Option<Connection>,
}

impl SqliteDataSource {
    pub fn init() -> Self {
        Self { conn: None }
    }
}

impl DataSourceDriver for SqliteDataSource {
    async fn connect(&mut self, sconn: String) -> Result<(), String> {
        let conn = sqlite::open(sconn)
            .map_err(|e| format!("Sqlite connection failed: {}", e.to_string()))?;

        self.conn = Some(conn);

        Ok(())
    }

    async fn fetch(&mut self) -> Result<(), String> {
        let conn = self
            .conn
            .as_ref()
            .ok_or("Connection not established".to_string())?;

        let query = "SELECT * FROM users";
        let mut statement = conn
            .prepare(query)
            .map_err(|e| format!("Prepare statement failed: {}", e.to_string()))?;

        while let Ok(State::Row) = statement.next() {
            println!(
                "name = {}",
                statement
                    .read::<String, _>("name")
                    .map_err(|e| format!("Read column name failed: {}", e.to_string()))?
            );
            println!(
                "age = {}",
                statement
                    .read::<i64, _>("age")
                    .map_err(|e| format!("Read column age failed: {}", e.to_string()))?
            );
        }

        Ok(())
    }
}
