//! Base SQLite locale.
//!
//! - Un **thread dédié** possède l'unique connexion ; les autres composants
//!   lui envoient des tâches et attendent la réponse de façon asynchrone.
//!   Le thread de l'interface n'exécute donc jamais de requête.
//! - Migrations versionnées (`PRAGMA user_version`), chacune dans une
//!   transaction. Avant de migrer une base existante, une copie de sécurité
//!   est créée. Une base créée par une version plus récente est refusée,
//!   jamais modifiée.
//! - Les données volumineuses resteront hors de SQLite (fichiers gérés).

pub mod profiles;

use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;

use jiff::Timestamp;
use rusqlite::Connection;

use crate::paths::create_private_dir;

/// Migrations, dans l'ordre. Ne jamais modifier une migration publiée :
/// ajouter une nouvelle entrée.
const MIGRATIONS: &[&str] = &[include_str!("../../migrations/0001_profiles.sql")];

/// Version de schéma attendue par cette version de l'application.
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

/// Erreurs de la base de données.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Ouverture impossible.
    #[error("impossible d'ouvrir la base {path}")]
    Open {
        /// Fichier concerné.
        path: PathBuf,
        /// Erreur d'origine.
        #[source]
        source: rusqlite::Error,
    },
    /// La base a été créée par une version plus récente de TSSR Labs.
    #[error(
        "la base de données utilise le schéma {found}, plus récent que celui de cette \
         version ({supported}) ; mettez TSSR Labs à jour (aucune donnée n'a été modifiée)"
    )]
    NewerSchema {
        /// Version trouvée.
        found: i64,
        /// Version prise en charge.
        supported: i64,
    },
    /// Échec d'une migration (annulée).
    #[error("échec de la migration vers la version {version} (annulée)")]
    Migration {
        /// Version visée.
        version: i64,
        /// Erreur d'origine.
        #[source]
        source: rusqlite::Error,
    },
    /// Échec d'une copie de sécurité.
    #[error("impossible de créer la copie de sécurité {path}")]
    Backup {
        /// Fichier visé.
        path: PathBuf,
        /// Erreur d'origine.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Erreur SQLite.
    #[error("erreur SQLite")]
    Sqlite(#[from] rusqlite::Error),
    /// Donnée stockée incohérente.
    #[error("donnée invalide dans la base : {0}")]
    InvalidData(String),
    /// Le thread de la base est arrêté.
    #[error("la base de données est fermée")]
    Closed,
    /// Le thread de la base n'a pas pu être lancé.
    #[error("impossible de lancer le thread de la base de données")]
    Thread(#[source] std::io::Error),
}

type Job = Box<dyn FnOnce(&mut Connection) + Send>;

enum Message {
    Run(Job),
    Shutdown,
}

/// Accès partagé à la base (clonable). Les requêtes s'exécutent sur le
/// thread de la base.
#[derive(Debug, Clone)]
pub struct Database {
    sender: mpsc::Sender<Message>,
    backup_dir: Arc<PathBuf>,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Run(_) => "Run",
            Self::Shutdown => "Shutdown",
        })
    }
}

/// Propriétaire du thread de la base : à sa destruction (ou via
/// [`DatabaseThread::close`]), les tâches en attente sont terminées puis la
/// connexion est fermée proprement.
#[derive(Debug)]
pub struct DatabaseThread {
    sender: mpsc::Sender<Message>,
    handle: Option<JoinHandle<()>>,
}

impl DatabaseThread {
    /// Arrête le thread après les tâches déjà soumises et attend sa fin.
    pub fn close(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if let Some(handle) = self.handle.take() {
            // Si l'envoi échoue, le thread est déjà arrêté.
            let _ = self.sender.send(Message::Shutdown);
            if handle.join().is_err() {
                tracing::error!("le thread de la base s'est arrêté sur une panique");
            }
        }
    }
}

impl Drop for DatabaseThread {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Ouvre (ou crée) la base, applique les migrations et démarre son thread.
///
/// L'ouverture est synchrone : elle a lieu au démarrage, avant l'interface.
pub fn open(path: &Path, backup_dir: &Path) -> Result<(Database, DatabaseThread), DbError> {
    let (sender, receiver) = mpsc::channel::<Message>();
    let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<(), DbError>>(1);
    let db_path = path.to_path_buf();
    let thread_backup_dir = backup_dir.to_path_buf();

    let handle = std::thread::Builder::new()
        .name("tssrlabs-sqlite".into())
        .spawn(move || {
            let mut conn = match open_and_migrate(&db_path, &thread_backup_dir) {
                Ok(conn) => {
                    let _ = ready_tx.send(Ok(()));
                    conn
                }
                Err(err) => {
                    let _ = ready_tx.send(Err(err));
                    return;
                }
            };
            serve(&mut conn, &receiver);
            tracing::debug!("thread de la base arrêté");
        })
        .map_err(DbError::Thread)?;

    let mut thread = DatabaseThread {
        sender: sender.clone(),
        handle: Some(handle),
    };
    match ready_rx.recv() {
        Ok(Ok(())) => Ok((
            Database {
                sender,
                backup_dir: Arc::new(backup_dir.to_path_buf()),
            },
            thread,
        )),
        Ok(Err(err)) => {
            thread.shutdown();
            Err(err)
        }
        Err(_) => {
            thread.shutdown();
            Err(DbError::Closed)
        }
    }
}

fn serve(conn: &mut Connection, receiver: &mpsc::Receiver<Message>) {
    while let Ok(message) = receiver.recv() {
        match message {
            Message::Run(job) => {
                // Une panique dans une tâche ne doit pas arrêter le thread :
                // l'appelant reçoit `DbError::Closed` (réponse abandonnée).
                if std::panic::catch_unwind(AssertUnwindSafe(|| job(conn))).is_err() {
                    tracing::error!("panique dans une tâche de la base de données");
                }
            }
            Message::Shutdown => break,
        }
    }
}

impl Database {
    /// Exécute `task` sur le thread de la base et attend son résultat.
    pub async fn call<T, F>(&self, task: F) -> Result<T, DbError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, DbError> + Send + 'static,
    {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        let job: Job = Box::new(move |conn| {
            let _ = reply_tx.send(task(conn));
        });
        self.sender
            .send(Message::Run(job))
            .map_err(|_| DbError::Closed)?;
        reply_rx.await.map_err(|_| DbError::Closed)?
    }

    /// Crée une copie cohérente de la base dans le répertoire des copies de
    /// sécurité et renvoie son chemin.
    pub async fn backup(&self, reason: &str) -> Result<PathBuf, DbError> {
        let dir = self.backup_dir.as_ref().clone();
        let reason = reason.to_owned();
        self.call(move |conn| backup_to_dir(conn, &dir, &reason))
            .await
    }
}

fn open_and_migrate(path: &Path, backup_dir: &Path) -> Result<Connection, DbError> {
    let mut conn = Connection::open(path).map_err(|source| DbError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    configure(&conn)?;
    migrate(&mut conn, MIGRATIONS, backup_dir)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> Result<(), DbError> {
    conn.busy_timeout(Duration::from_secs(5))?;
    // WAL : lectures non bloquées par les écritures, meilleure résistance
    // aux interruptions.
    let mode: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
        tracing::warn!(%mode, "mode WAL indisponible pour la base");
    }
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

fn schema_version(conn: &Connection) -> Result<i64, DbError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

fn migrate(conn: &mut Connection, migrations: &[&str], backup_dir: &Path) -> Result<(), DbError> {
    let latest = migrations.len() as i64;
    let current = schema_version(conn)?;
    if current > latest {
        return Err(DbError::NewerSchema {
            found: current,
            supported: latest,
        });
    }
    if current == latest {
        return Ok(());
    }
    if current > 0 {
        let backup = backup_to_dir(conn, backup_dir, &format!("avant-migration-v{current}"))?;
        tracing::info!(from = current, to = latest, backup = %backup.display(), "copie de sécurité avant migration");
    }
    for (index, sql) in migrations.iter().enumerate().skip(current as usize) {
        let version = index as i64 + 1;
        let apply = |conn: &mut Connection| -> rusqlite::Result<()> {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()
        };
        apply(conn).map_err(|source| DbError::Migration { version, source })?;
        tracing::info!(version, "migration appliquée");
    }
    Ok(())
}

fn backup_to_dir(conn: &Connection, dir: &Path, reason: &str) -> Result<PathBuf, DbError> {
    let backup_error =
        |path: &Path, source: Box<dyn std::error::Error + Send + Sync>| DbError::Backup {
            path: path.to_path_buf(),
            source,
        };
    create_private_dir(dir).map_err(|e| backup_error(dir, Box::new(e)))?;

    let now = Timestamp::now();
    let stamp = format!(
        "{}-{:03}",
        now.strftime("%Y%m%d-%H%M%S"),
        now.as_millisecond().rem_euclid(1000)
    );
    let label: String = reason
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let mut path = dir.join(format!("tssrlabs-{label}-{stamp}.sqlite3"));
    let mut counter = 2;
    while path.exists() {
        path = dir.join(format!("tssrlabs-{label}-{stamp}-{counter}.sqlite3"));
        counter += 1;
    }

    let target = path
        .to_str()
        .ok_or_else(|| backup_error(&path, "chemin non UTF-8".into()))?
        .to_owned();
    conn.execute("VACUUM INTO ?1", [target])
        .map_err(|e| backup_error(&path, Box::new(e)))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap();
        stmt.query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    fn backups(dir: &Path) -> usize {
        std::fs::read_dir(dir).map(|d| d.count()).unwrap_or(0)
    }

    #[test]
    fn real_migrations_apply_on_fresh_database() {
        let tmp = tempfile::tempdir().unwrap();
        let mut conn = Connection::open(tmp.path().join("db")).unwrap();
        migrate(&mut conn, MIGRATIONS, &tmp.path().join("b")).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
        assert_eq!(
            tables(&conn),
            vec!["app_state", "profile_settings", "profiles"]
        );
        assert_eq!(
            backups(&tmp.path().join("b")),
            0,
            "pas de copie pour une base neuve"
        );
    }

    #[test]
    fn upgrade_backs_up_then_applies_missing_migrations() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tmp.path().join("copies");
        let v1: &[&str] = &["CREATE TABLE a (x INTEGER);"];
        let v2: &[&str] = &["CREATE TABLE a (x INTEGER);", "CREATE TABLE b (y INTEGER);"];

        let mut conn = Connection::open(tmp.path().join("db")).unwrap();
        migrate(&mut conn, v1, &backup_dir).unwrap();
        conn.execute("INSERT INTO a VALUES (42)", []).unwrap();
        migrate(&mut conn, v2, &backup_dir).unwrap();

        assert_eq!(schema_version(&conn).unwrap(), 2);
        assert_eq!(tables(&conn), vec!["a", "b"]);
        assert_eq!(backups(&backup_dir), 1);

        // La copie contient l'état d'avant migration.
        let entry = std::fs::read_dir(&backup_dir)
            .unwrap()
            .next()
            .unwrap()
            .unwrap();
        let copy = Connection::open(entry.path()).unwrap();
        assert_eq!(schema_version(&copy).unwrap(), 1);
        let x: i64 = copy.query_row("SELECT x FROM a", [], |r| r.get(0)).unwrap();
        assert_eq!(x, 42);
    }

    #[test]
    fn failing_migration_is_rolled_back() {
        let tmp = tempfile::tempdir().unwrap();
        let broken: &[&str] = &["CREATE TABLE a (x INTEGER); CREATE TABLE a (x INTEGER);"];
        let mut conn = Connection::open(tmp.path().join("db")).unwrap();
        let err = migrate(&mut conn, broken, tmp.path()).unwrap_err();
        assert!(matches!(err, DbError::Migration { version: 1, .. }));
        assert_eq!(schema_version(&conn).unwrap(), 0);
        assert!(tables(&conn).is_empty());
    }

    #[test]
    fn newer_schema_is_refused_without_modification() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("db");
        {
            let conn = Connection::open(&path).unwrap();
            conn.pragma_update(None, "user_version", 99).unwrap();
        }
        let err = open(&path, &tmp.path().join("b")).unwrap_err();
        assert!(
            matches!(err, DbError::NewerSchema { found: 99, supported } if supported == SCHEMA_VERSION)
        );
        let conn = Connection::open(&path).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), 99);
    }

    #[tokio::test]
    async fn calls_run_on_db_thread_and_survive_panics() {
        let tmp = tempfile::tempdir().unwrap();
        let (db, thread) = open(&tmp.path().join("db"), &tmp.path().join("b")).unwrap();

        let version = db.call(|conn| schema_version(conn)).await.unwrap();
        assert_eq!(version, SCHEMA_VERSION);

        let panicked = db
            .call(|_| -> Result<(), DbError> { panic!("tâche défaillante") })
            .await;
        assert!(matches!(panicked, Err(DbError::Closed)));

        // Le thread fonctionne toujours.
        assert!(db.call(|conn| schema_version(conn)).await.is_ok());

        thread.close();
        assert!(matches!(
            db.call(|conn| schema_version(conn)).await,
            Err(DbError::Closed)
        ));
    }

    #[tokio::test]
    async fn backup_creates_distinct_readable_copies() {
        let tmp = tempfile::tempdir().unwrap();
        let backup_dir = tmp.path().join("copies");
        let (db, _thread) = open(&tmp.path().join("db"), &backup_dir).unwrap();

        let first = db.backup("test/../évasion").await.unwrap();
        let second = db.backup("test/../évasion").await.unwrap();
        assert_ne!(first, second);
        for copy in [&first, &second] {
            assert_eq!(copy.parent(), Some(backup_dir.as_path()), "{copy:?}");
            let conn = Connection::open(copy).unwrap();
            assert_eq!(schema_version(&conn).unwrap(), SCHEMA_VERSION);
        }
    }
}
