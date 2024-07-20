mod models;
mod schema;

use crate::domain::{ChannelEntryId, ChannelInfo};
use anyhow::{anyhow, bail, Context};
use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::{
    fs,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl Db {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = Self::open_connection(path.as_ref())?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
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

    pub async fn get_channels(&self) -> Vec<(ChannelEntryId, ChannelInfo)> {
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            use schema::channels;

            let mut conn = conn.lock().expect("connection shouldn't be poisoned");

            channels::table
                .select(models::Channel::as_select())
                .load(&mut *conn)
                .expect("database operations should be successful")
                .into_iter()
                .map(|channel| (ChannelEntryId(channel.id), channel.into()))
                .collect()
        })
        .await
        .expect("database queries shouldn't panic")
    }

    pub async fn new_channel(&self, info: &ChannelInfo) -> ChannelEntryId {
        let info: models::NewChannel = info.to_owned().into();
        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            use schema::channels;

            let mut conn = conn.lock().expect("connection shouldn't be poisoned");

            let id = diesel::insert_into(channels::table)
                .values(info)
                .returning(channels::id)
                .get_result::<i32>(&mut *conn)
                .expect("database operations should be successful");

            ChannelEntryId(id)
        })
        .await
        .expect("database queries shouldn't panic")
    }

    pub async fn update_channel(&self, id: ChannelEntryId, info: &ChannelInfo) {
        let row_id: i32 = id.0;
        let info: models::NewChannel = info.to_owned().into();

        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            use schema::channels;

            let mut conn = conn.lock().expect("connection shouldn't be poisoned");

            diesel::update(channels::table)
                .filter(channels::id.eq(row_id))
                .set(info)
                .execute(&mut *conn)
                .expect("database operations should be successful");
        })
        .await
        .expect("database queries shouldn't panic")
    }

    pub async fn remove_channel(&self, id: ChannelEntryId) {
        let row_id: i32 = id.0;

        let conn = self.conn.clone();

        tokio::task::spawn_blocking(move || {
            use schema::channels;

            let mut conn = conn.lock().expect("connection shouldn't be poisoned");

            diesel::delete(channels::table)
                .filter(channels::id.eq(row_id))
                .execute(&mut *conn)
                .expect("database operations should be successful");
        })
        .await
        .expect("database queries shouldn't panic")
    }
}
