use crate::apis::utils_api::sql;
use crate::Configuration;
use chrono::{DateTime, Utc};
use std::path::Path;
use serde_repr::Deserialize_repr;
use tokio::fs;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Deserialize_repr,PartialEq)]
#[repr(u8)]
pub enum MigrationType {
    Up = 0,
    Down
}

impl fmt::Display for MigrationType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Up => write!(f, "Up"),
            Self::Down => write!(f, "Down")
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Migration {
    name: String,
    applied: Option<i32>,
    checksum: i64,
    typ: MigrationType,
    #[serde(skip)]
    contents: String,
}

impl Migration {
    fn new(name: String, contents: String, typ: MigrationType) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write(contents.as_bytes());
        let checksum = hasher.finish() as i64;
        Self{name, contents, checksum, typ, applied: None}
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn checksum(&self) -> i64 {
        self.checksum
    }

    pub fn typ(&self) -> MigrationType {
        self.typ.clone()
    }

    pub fn applied(&self) -> Option<DateTime<Utc>> {
        self.applied.map(|t| DateTime::<Utc>::from_timestamp(t as i64, 0).expect("assert: migration timestamp should be correctly created"))
    }
}

impl fmt::Display for Migration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.applied.is_some() {
            write!(f, "{}({}, {})", self.name(), self.typ(), self.applied().unwrap().format("%Y-%m-%d %H:%M:%S"))
        } else {
            write!(f, "{}({}, unapplied)", self.name(), self.typ())
        }
    }
}

async fn read_migration_script(path: &Path) -> Result<String, String> {
    let contents = fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    let contents: Vec<&str> = contents.lines().filter(|l| {
        let line = l.trim();
        !(line.starts_with("--") || line.is_empty())
    }).map(|l| l.trim()).collect();
    let contents = contents.join("\n");
    if contents.is_empty() {
        return Err(format!("migration `{}` cannot be empty", path.display()));
    }
    let mut iter = contents.split(';').peekable();
    while let Some(statement) = iter.next() {
        if iter.peek().is_some() && statement.trim().is_empty() {
            return Err(format!("migration `{}` contains empty statement: probably an extra `;` is present", path.display()));
        }
    }
    Ok(contents)
}

pub struct Migrator {
    configuration: Configuration,
    directory: String,
}

impl Migrator {
    pub fn new(base_path: String, migrations_dir: &str) -> Self {
        let configuration = Configuration {
            base_path,
            user_agent: Some("manticoresearch-rs/migrator".to_string()),
            client: reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build(),
            basic_auth: None,
            oauth_access_token: None,
            bearer_access_token: None,
            api_key: None,
        };
        Self { configuration, directory: migrations_dir.strip_suffix("/").unwrap_or(migrations_dir).to_string() }
    }

    pub fn directory(&self) -> &str {
        &self.directory
    }

    async fn sql(&self, query: &str) -> Result<Option<serde_json::Value>, String> {
        let mut data = match sql(
            &self.configuration, 
            query, 
            Some(true))
        .await.map_err(|e| format!("{e:?}")) {
            Ok(data) => data,
            Err(message) => {
                if message.contains("unknown local table(s) '_migrations' in search request") {
                    return Ok(None);
                } else {
                    return Err(message);
                }
            }
        };
        let error: String = serde_json::from_value(data[0]["error"].take()).map_err(|e| e.to_string())?;
        if !error.is_empty() {
            tracing::error!("{}", error);
        }
        let warning: String = serde_json::from_value(data[0]["warning"].take()).map_err(|e| e.to_string())?;
        if !warning.is_empty() {
            tracing::warn!("{}", warning);
        }
        Ok(Some(data[0]["data"].take()))
    }

    async fn initialize(&mut self) -> Result<(), String> {
        self.sql(
            "CREATE TABLE IF NOT EXISTS _migrations(name string, applied timestamp, checksum bigint, typ bit(3))",
        )
        .await?;
        Ok(())
    }

    async fn list_dir(&self) -> Result<Vec<(Migration, Migration)>, String> {
        let mut dirs = vec![];
        let mut directory = fs::read_dir(&self.directory).await.map_err(|e| e.to_string())?;
        loop {
            match directory.next_entry().await {
                Ok(Some(dir)) => {
                    let name = dir.file_name().into_string().map_err(|e| format!("{e:?}"))?;
                    if name.len() != 14 { // `yyyymmddhhmmss`
                        continue;
                    }
                    dirs.push((
                        name.parse::<u64>().map_err(|e| format!("Migration subdirectory should be named `yyyymmddhhmmss`: {e}"))?,
                        dir
                    ));
                },
                Ok(None) => break,
                Err(e) => return Err(e.to_string()),
            }
        }
        dirs.sort_unstable_by_key(|d| d.0);
        let mut migrations = Vec::with_capacity(dirs.len());

        for (name, d) in dirs.into_iter() {
            let up = d.path().join("up.sql");
            let contents = read_migration_script(&up).await?;
            let up = Migration::new(name.to_string(), contents, MigrationType::Up);
            let down = d.path().join("down.sql");
            let contents = read_migration_script(&down).await?;
            let down = Migration::new(name.to_string(), contents, MigrationType::Down);
            migrations.push((up, down));
        }
        Ok(migrations)
    }

    pub async fn history(&mut self) -> Result<Vec<Migration>, String> {
        let data_option = self.sql(
            "
            SELECT name, applied, checksum, typ
            FROM _migrations
            ORDER BY applied ASC;
            ")
        .await?;
        if let Some(data) = data_option {
            let migrations: Vec<Migration> = serde_json::from_value(data).map_err(|e| e.to_string())?;
            Ok(migrations)
        } else {
            Ok(vec![])
        }
    }

    async fn last_applied_migrations(&mut self) -> Result<HashMap<String, Migration>, String> {
        let applied_migrations_option = self.sql(
            "
            SELECT name, applied, checksum, typ
            FROM _migrations
            ORDER BY applied ASC;
            ")
        .await?;
        if let Some(applied_migrations) = applied_migrations_option {
            let applied_migrations: Vec<Migration> = serde_json::from_value(applied_migrations).map_err(|e| e.to_string())?;
            let last_applied_migrations: HashMap<String, Migration> = applied_migrations.into_iter().fold(HashMap::new(), |mut map, m| {
                map.insert(m.name().to_string(), m);
                map
            });
            Ok(last_applied_migrations)
        } else {
            Ok(HashMap::new())
        }
    }

    pub async fn list(&mut self) -> Result<Vec<Migration>, String> {
        let migrations = self.list_dir().await?;
        let last_applied_migrations = self.last_applied_migrations().await?;
        let mut actual_migrations = vec![];
        for (mut up_migration, down_migration) in migrations.into_iter() {
            if let Some(applied) = last_applied_migrations.get(up_migration.name()) {
                if applied.typ == MigrationType::Up && up_migration.checksum() != applied.checksum() {
                    return Err(format!("checksum of the migration `{}` has changed since it was applied", applied));
                }
                if applied.typ == MigrationType::Down && down_migration.checksum() != applied.checksum() {
                    return Err(format!("checksum of the migration `{}` has changed since it was applied", applied));
                }
                if applied.typ == MigrationType::Down {
                    continue;
                }
                up_migration.applied = applied.applied;
            }
            actual_migrations.push(up_migration);
        } 
        Ok(actual_migrations)
    }

    pub async fn up(&mut self, fake: bool) -> Result<Vec<Migration>, String> {
        self.initialize().await?;
        let last_applied_migrations = self.last_applied_migrations().await?;
        let migrations = self.list_dir().await?;
        let mut applied_migrations = vec![];
        for (migration, _) in migrations.into_iter() {
            if let Some(applied) = last_applied_migrations.get(migration.name()) {
                if applied.typ == MigrationType::Up {
                    continue;
                }
            }
            tracing::info!("applying `{}`...", migration);
            let mut iter = migration.contents.split(';').peekable();
            while let Some(statement) = iter.next() {
                if iter.peek().is_none(){
                    break;
                }
                if fake {
                    println!("{statement}");
                    continue;
                }
                self.sql(statement).await?;
            }
            if fake {
                continue;
            }
            self.sql(
                &format!("INSERT INTO _migrations(name, applied, checksum, typ) VALUES ('{}', {}, {}, {})",
                    migration.name(), Utc::now().timestamp(), migration.checksum(), migration.typ() as u8
                )
            )
            .await?;
            tracing::info!("applied `{}`", migration);
            applied_migrations.push(migration);
        }
        Ok(applied_migrations)
    }

    pub async fn down(&mut self, fake: bool, name: &str) -> Result<Vec<Migration>, String> {
        let last_applied_migrations = self.last_applied_migrations().await?;
        if let Some(applied) = last_applied_migrations.get(name) {
            if applied.typ == MigrationType::Down {
                return Err(format!("migration with the name `{name}` has been already rolled back"));
            }
        } else {
            return Err(format!("migration with the name `{name}` is not found among applied migrations"));
        }

        let migrations = self.list_dir().await?;
        let mut rolled_backed_migrations = vec![];
        for (mut up_migration, down_migration) in migrations.into_iter().rev() {
            if let Some(applied) = last_applied_migrations.get(up_migration.name()) {
                if applied.typ == MigrationType::Down && down_migration.checksum() != applied.checksum() {
                    return Err(format!("checksum of the migration `{}` has changed since it was applied", applied));
                }
                if applied.typ == MigrationType::Down {
                    continue;
                }
                up_migration.applied = applied.applied;
            } else {
                continue;
            }
            tracing::info!("rolling back `{}`...", up_migration);
            let mut iter = down_migration.contents.split(';').peekable();
            while let Some(statement) = iter.next() {
                if iter.peek().is_none(){
                    break;
                }
                if fake {
                    println!("{statement}");
                    continue;
                }
                self.sql(statement).await?;
            }
            if fake {
                continue;
            }
            self.sql(
                &format!("INSERT INTO _migrations(name, applied, checksum, typ) VALUES ('{}', {}, {}, {})",
                down_migration.name(), Utc::now().timestamp(), down_migration.checksum(), down_migration.typ() as u8
                )
            )
            .await?;
            tracing::info!("rolled back `{}`", up_migration);
            if down_migration.name == name {
                rolled_backed_migrations.push(down_migration);
                break;
            }
            rolled_backed_migrations.push(down_migration);
            
        }
        Ok(rolled_backed_migrations)
    }

    pub async fn add(&self) -> Result<String, String> {
        let dir_name = Utc::now().format("%Y%m%d%H%M%S");
        let base_name = format!("{}/{dir_name}", self.directory);
        fs::create_dir_all(&base_name).await.map_err(|e| format!("{e:?}"))?;
        fs::write(format!("{base_name}/up.sql"), "--Put a new migration here. Prefer idempotence, i.e. `CREATE TABLE IF NOT EXISTS`").await.map_err(|e| format!("{e:?}"))?;
        fs::write(format!("{base_name}/down.sql"), "--Put a rollback migration here. Prefer idempotence, i.e. `DROP TABLE IF EXISTS`").await.map_err(|e| format!("{e:?}"))?;
        Ok(base_name)
    }
}
