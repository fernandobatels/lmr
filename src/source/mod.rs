//! Data sources drivers

use crate::config::{ConfigSource, SourceType};

pub mod sqlite;

/// Data source driver definitions
pub trait DataSourceDriver {
    // Establish the connection and prepare for fetch
    async fn connect(&mut self, conn: String) -> Result<(), String>;

    // Query and fetch the data
    async fn fetch(&mut self) -> Result<(), String>;
}

fn get_driver(kind: SourceType) -> Result<impl DataSourceDriver, String> {
    match kind {
        SourceType::Sqlite => Ok(sqlite::SqliteDataSource::init()),
        _ => Err("Not supported kind".to_string()),
    }
}

/// Query and fetch the data from the database
pub async fn fetch(source: ConfigSource) -> Result<(), String> {
    let mut driver = get_driver(source.kind)?;

    driver.connect(source.conn).await?;

    driver.fetch().await?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use crate::config::{ConfigSource, SourceType};

    #[tokio::test]
    async fn fetch() -> Result<(), String> {
        let query = "
            drop table if exists users;
            CREATE TABLE users (name TEXT, age INTEGER);
            INSERT INTO users VALUES ('Alice', 42);
            INSERT INTO users VALUES ('Bob', 69);
        ";
        sqlite::Connection::open_with_flags(
            "/tmp/test-lmr.db",
            sqlite::OpenFlags::new().with_create().with_read_write(),
        )
        .unwrap()
        .execute(query)
        .unwrap();

        let source = ConfigSource {
            conn: "/tmp/test-lmr.db".to_string(),
            kind: SourceType::Sqlite,
        };

        super::fetch(source).await?;

        Ok(())
    }
}
