use anyhow::{anyhow, bail, Context};
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{fs, io::Write, path::Path};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct Db {
    conn: SqliteConnection,
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Ok(Self {
            conn: Self::open_connection(path.as_ref())?,
        })
    }

    fn open_connection(path: &Path) -> anyhow::Result<SqliteConnection> {
        let Some(path) = path.to_str() else {
            bail!("path '{}' contains non-unicode characters", path.display());
        };

        // Создание бэкапа на случай, если новая версия окажется очень косячной.
        Self::backup_database(Path::new(path)).context("backuping database")?;

        // Создание базы данных и проверка, что её можно читать и писать.
        fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .read(true)
            .open(path)
            .with_context(|| format!("opening database '{path}'"))?
            .write(b"")
            .with_context(|| format!("writing to database '{path}'"))?;

        let mut connection = SqliteConnection::establish(path)
            .with_context(|| format!("establish connection to '{path}'"))?;

        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|err| anyhow!("migrations error: {err}"))?;

        Ok(connection)
    }

    fn backup_database(path: &Path) -> anyhow::Result<()> {
        let dpath = path.display();

        let db_exists = path
            .try_exists()
            .with_context(|| format!("checking database at '{dpath}'"))?;

        if !db_exists {
            return Ok(());
        }

        if !path.is_file() {
            bail!("'{dpath}' is not file");
        }

        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            bail!("file '{dpath}' doesn't have utf-8 file extension");
        };

        let backup_path = path.with_extension(format!("{extension}.bak"));
        let dbackup_path = backup_path.display();

        fs::copy(path, &backup_path)
            .with_context(|| format!("copying file from '{dpath}' to '{dbackup_path}'"))?;

        log::info!("Successfully backup database from '{dpath}' to '{dbackup_path}'");

        Ok(())
    }
}
