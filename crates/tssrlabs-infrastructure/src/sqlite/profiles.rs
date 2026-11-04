//! Implémentation SQLite des ports de profils et de copie de sécurité.

use std::path::PathBuf;

use futures_util::future::BoxFuture;
use jiff::Timestamp;
use rusqlite::{Connection, ErrorCode, OptionalExtension, params};
use tssrlabs_application::ports::{ProfileRepository, RepoResult, RepositoryError, SafetyBackup};
use tssrlabs_domain::preferences::{Preferences, ThemeChoice};
use tssrlabs_domain::profile::{Profile, ProfileId, ProfileName};

use super::{Database, DbError};

const ACTIVE_PROFILE_KEY: &str = "active_profile";
const THEME_KEY: &str = "theme";

/// Profils stockés dans SQLite.
#[derive(Debug, Clone)]
pub struct SqliteProfileRepository {
    db: Database,
}

impl SqliteProfileRepository {
    /// Crée le dépôt.
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

/// Copies de sécurité de la base SQLite.
#[derive(Debug, Clone)]
pub struct SqliteSafetyBackup {
    db: Database,
}

impl SqliteSafetyBackup {
    /// Crée l'adaptateur.
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn to_repository_error(error: DbError) -> RepositoryError {
    match error {
        DbError::Sqlite(rusqlite::Error::SqliteFailure(e, _))
            if e.code == ErrorCode::ConstraintViolation =>
        {
            RepositoryError::Conflict
        }
        DbError::Sqlite(rusqlite::Error::SqliteFailure(e, _))
            if matches!(e.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) =>
        {
            RepositoryError::Unavailable("base verrouillée par un autre processus".into())
        }
        DbError::Closed => RepositoryError::Unavailable(DbError::Closed.to_string()),
        other => {
            tracing::error!(error = %error_chain(&other), "erreur de la base de données");
            RepositoryError::Backend(error_chain(&other))
        }
    }
}

fn error_chain(err: &dyn std::error::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(" : ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

fn timestamp(ms: i64) -> Result<Timestamp, DbError> {
    Timestamp::from_millisecond(ms).map_err(|e| DbError::InvalidData(format!("date {ms} : {e}")))
}

fn row_to_profile(
    id: i64,
    name: String,
    created: i64,
    used: Option<i64>,
) -> Result<Profile, DbError> {
    Ok(Profile {
        id: ProfileId::new(id),
        name: ProfileName::parse(&name)
            .map_err(|e| DbError::InvalidData(format!("nom du profil {id} : {e}")))?,
        created_at: timestamp(created)?,
        last_used_at: used.map(timestamp).transpose()?,
    })
}

fn list_profiles(conn: &mut Connection) -> Result<Vec<Profile>, DbError> {
    let mut stmt =
        conn.prepare_cached("SELECT id, name, created_at, last_used_at FROM profiles ORDER BY id")?;
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<i64>>(3)?,
        ))
    })?;
    let mut profiles = Vec::new();
    for row in rows {
        let (id, name, created, used) = row?;
        profiles.push(row_to_profile(id, name, created, used)?);
    }
    Ok(profiles)
}

fn require_changed(changed: usize) -> Result<(), DbError> {
    if changed == 0 {
        Err(DbError::InvalidData("introuvable".into()))
    } else {
        Ok(())
    }
}

/// Distingue « introuvable » des autres données invalides.
fn not_found_aware(error: DbError) -> RepositoryError {
    match error {
        DbError::InvalidData(ref message) if message == "introuvable" => RepositoryError::NotFound,
        other => to_repository_error(other),
    }
}

impl ProfileRepository for SqliteProfileRepository {
    fn list(&self) -> BoxFuture<'_, RepoResult<Vec<Profile>>> {
        Box::pin(async move {
            self.db
                .call(list_profiles)
                .await
                .map_err(to_repository_error)
        })
    }

    fn insert(&self, name: ProfileName) -> BoxFuture<'_, RepoResult<Profile>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let now = Timestamp::now();
                    conn.execute(
                        "INSERT INTO profiles (name, name_key, created_at) VALUES (?1, ?2, ?3)",
                        params![name.as_str(), name.key(), now.as_millisecond()],
                    )?;
                    Ok(Profile {
                        id: ProfileId::new(conn.last_insert_rowid()),
                        name,
                        created_at: timestamp(now.as_millisecond())?,
                        last_used_at: None,
                    })
                })
                .await
                .map_err(to_repository_error)
        })
    }

    fn rename(&self, id: ProfileId, name: ProfileName) -> BoxFuture<'_, RepoResult<()>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let changed = conn.execute(
                        "UPDATE profiles SET name = ?1, name_key = ?2 WHERE id = ?3",
                        params![name.as_str(), name.key(), id.get()],
                    )?;
                    require_changed(changed)
                })
                .await
                .map_err(not_found_aware)
        })
    }

    fn delete(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<()>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let tx = conn.transaction()?;
                    // Les préférences suivent par ON DELETE CASCADE.
                    let changed = tx.execute("DELETE FROM profiles WHERE id = ?1", [id.get()])?;
                    require_changed(changed)?;
                    tx.execute(
                        "DELETE FROM app_state WHERE key = ?1 AND value = ?2",
                        params![ACTIVE_PROFILE_KEY, id.get().to_string()],
                    )?;
                    tx.commit()?;
                    Ok(())
                })
                .await
                .map_err(not_found_aware)
        })
    }

    fn active_profile(&self) -> BoxFuture<'_, RepoResult<Option<ProfileId>>> {
        Box::pin(async move {
            self.db
                .call(|conn| {
                    let value: Option<String> = conn
                        .query_row(
                            "SELECT value FROM app_state WHERE key = ?1",
                            [ACTIVE_PROFILE_KEY],
                            |row| row.get(0),
                        )
                        .optional()?;
                    let Some(id) = value.and_then(|v| v.parse::<i64>().ok()) else {
                        return Ok(None);
                    };
                    let exists: bool = conn.query_row(
                        "SELECT EXISTS (SELECT 1 FROM profiles WHERE id = ?1)",
                        [id],
                        |row| row.get(0),
                    )?;
                    Ok(exists.then_some(ProfileId::new(id)))
                })
                .await
                .map_err(to_repository_error)
        })
    }

    fn set_active_profile(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<()>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let tx = conn.transaction()?;
                    let changed = tx.execute(
                        "UPDATE profiles SET last_used_at = ?1 WHERE id = ?2",
                        params![Timestamp::now().as_millisecond(), id.get()],
                    )?;
                    require_changed(changed)?;
                    tx.execute(
                        "INSERT INTO app_state (key, value) VALUES (?1, ?2)
                         ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                        params![ACTIVE_PROFILE_KEY, id.get().to_string()],
                    )?;
                    tx.commit()?;
                    Ok(())
                })
                .await
                .map_err(not_found_aware)
        })
    }

    fn load_preferences(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<Preferences>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let theme: Option<String> = conn
                        .query_row(
                            "SELECT value FROM profile_settings WHERE profile_id = ?1 AND key = ?2",
                            params![id.get(), THEME_KEY],
                            |row| row.get(0),
                        )
                        .optional()?;
                    let theme = match theme.as_deref().map(|v| (v, ThemeChoice::parse(v))) {
                        Some((_, Some(theme))) => theme,
                        Some((value, None)) => {
                            tracing::warn!(value, "thème enregistré inconnu ; thème par défaut");
                            ThemeChoice::default()
                        }
                        None => ThemeChoice::default(),
                    };
                    Ok(Preferences { theme })
                })
                .await
                .map_err(to_repository_error)
        })
    }

    fn save_preferences(
        &self,
        id: ProfileId,
        preferences: Preferences,
    ) -> BoxFuture<'_, RepoResult<()>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    conn.execute(
                        "INSERT INTO profile_settings (profile_id, key, value) VALUES (?1, ?2, ?3)
                         ON CONFLICT (profile_id, key) DO UPDATE SET value = excluded.value",
                        params![id.get(), THEME_KEY, preferences.theme.as_str()],
                    )
                    .map_err(|e| match e {
                        rusqlite::Error::SqliteFailure(ref f, _)
                            if f.code == ErrorCode::ConstraintViolation =>
                        {
                            // Clé étrangère : le profil n'existe plus.
                            DbError::InvalidData("introuvable".into())
                        }
                        other => DbError::Sqlite(other),
                    })?;
                    Ok(())
                })
                .await
                .map_err(not_found_aware)
        })
    }
}

impl SafetyBackup for SqliteSafetyBackup {
    fn backup(&self, reason: &str) -> BoxFuture<'_, RepoResult<PathBuf>> {
        let reason = reason.to_owned();
        Box::pin(async move { self.db.backup(&reason).await.map_err(to_repository_error) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::{DatabaseThread, open};
    use std::sync::Arc;
    use tssrlabs_application::profiles::{ProfileError, ProfileService};

    struct Fixture {
        repo: SqliteProfileRepository,
        service: ProfileService,
        backup_dir: PathBuf,
        _thread: DatabaseThread,
        _tmp: tempfile::TempDir,
    }

    fn fixture() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tmp.path().join("copies");
        let (db, thread) = open(&tmp.path().join("tssrlabs.sqlite3"), &backup_dir).unwrap();
        let repo = SqliteProfileRepository::new(db.clone());
        let service = ProfileService::new(
            Arc::new(repo.clone()),
            Arc::new(SqliteSafetyBackup::new(db)),
        );
        Fixture {
            repo,
            service,
            backup_dir,
            _thread: thread,
            _tmp: tmp,
        }
    }

    fn name(value: &str) -> ProfileName {
        ProfileName::parse(value).unwrap()
    }

    #[tokio::test]
    async fn insert_list_and_unique_names() {
        let f = fixture();
        let alice = f.repo.insert(name("Alice")).await.unwrap();
        assert_eq!(
            f.repo.insert(name("ALICE")).await,
            Err(RepositoryError::Conflict)
        );
        let profiles = f.repo.list().await.unwrap();
        assert_eq!(profiles, vec![alice]);
    }

    #[tokio::test]
    async fn rename_and_missing_profile() {
        let f = fixture();
        let alice = f.repo.insert(name("Alice")).await.unwrap();
        f.repo.insert(name("Bob")).await.unwrap();
        assert_eq!(
            f.repo.rename(alice.id, name("bob")).await,
            Err(RepositoryError::Conflict)
        );
        // Changer seulement la casse de son propre nom est permis.
        f.repo.rename(alice.id, name("ALICE")).await.unwrap();
        assert_eq!(
            f.repo.rename(ProfileId::new(999), name("Zoé")).await,
            Err(RepositoryError::NotFound)
        );
    }

    #[tokio::test]
    async fn active_profile_round_trip_and_cleared_on_delete() {
        let f = fixture();
        let alice = f.repo.insert(name("Alice")).await.unwrap();
        assert_eq!(f.repo.active_profile().await.unwrap(), None);
        f.repo.set_active_profile(alice.id).await.unwrap();
        assert_eq!(f.repo.active_profile().await.unwrap(), Some(alice.id));
        assert!(f.repo.list().await.unwrap()[0].last_used_at.is_some());

        f.repo.delete(alice.id).await.unwrap();
        assert_eq!(f.repo.active_profile().await.unwrap(), None);
        assert_eq!(
            f.repo.delete(alice.id).await,
            Err(RepositoryError::NotFound)
        );
        assert_eq!(
            f.repo.set_active_profile(alice.id).await,
            Err(RepositoryError::NotFound)
        );
    }

    #[tokio::test]
    async fn preferences_round_trip_and_cascade() {
        let f = fixture();
        let alice = f.repo.insert(name("Alice")).await.unwrap();
        assert_eq!(
            f.repo.load_preferences(alice.id).await.unwrap(),
            Preferences::default()
        );
        let prefs = Preferences {
            theme: ThemeChoice::Colorblind,
        };
        f.repo.save_preferences(alice.id, prefs).await.unwrap();
        assert_eq!(f.repo.load_preferences(alice.id).await.unwrap(), prefs);

        f.repo.delete(alice.id).await.unwrap();
        assert_eq!(
            f.repo.save_preferences(alice.id, prefs).await,
            Err(RepositoryError::NotFound)
        );
    }

    #[tokio::test]
    async fn unknown_stored_theme_falls_back_to_default() {
        let f = fixture();
        let alice = f.repo.insert(name("Alice")).await.unwrap();
        let id = alice.id.get();
        f.repo
            .db
            .call(move |conn| {
                conn.execute(
                    "INSERT INTO profile_settings VALUES (?1, 'theme', 'arc-en-ciel')",
                    [id],
                )?;
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(
            f.repo.load_preferences(alice.id).await.unwrap(),
            Preferences::default()
        );
    }

    #[tokio::test]
    async fn service_end_to_end_with_safety_backup() {
        let f = fixture();
        let snapshot = f.service.create("Alice").await.unwrap();
        let alice = snapshot.active.unwrap().profile;

        assert_eq!(
            f.service.delete(alice.id, "alice").await,
            Err(ProfileError::ConfirmationMismatch)
        );
        let deletion = f.service.delete(alice.id, "Alice").await.unwrap();
        assert!(deletion.snapshot.profiles.is_empty());
        assert!(deletion.backup.starts_with(&f.backup_dir));

        // La copie de sécurité contient encore le profil supprimé.
        let copy = Connection::open(&deletion.backup).unwrap();
        let count: i64 = copy
            .query_row("SELECT COUNT(*) FROM profiles", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn data_persists_across_reopen() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("db");
        {
            let (db, thread) = open(&path, tmp.path()).unwrap();
            let repo = SqliteProfileRepository::new(db);
            let alice = repo.insert(name("Alice")).await.unwrap();
            repo.set_active_profile(alice.id).await.unwrap();
            thread.close();
        }
        let (db, _thread) = open(&path, tmp.path()).unwrap();
        let repo = SqliteProfileRepository::new(db);
        assert_eq!(repo.list().await.unwrap().len(), 1);
        assert!(repo.active_profile().await.unwrap().is_some());
    }
}
