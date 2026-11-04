//! Implémentation SQLite des tentatives d'exercices.

use futures_util::future::BoxFuture;
use jiff::Timestamp;
use rusqlite::{ErrorCode, params};
use tssrlabs_application::exercises::ProgressMap;
use tssrlabs_application::ports::{AttemptRepository, RepoResult, RepositoryError};
use tssrlabs_domain::content::ContentId;
use tssrlabs_domain::exercise::Outcome;
use tssrlabs_domain::profile::ProfileId;

use super::{Database, DbError};

/// Tentatives stockées dans SQLite.
#[derive(Debug, Clone)]
pub struct SqliteAttemptRepository {
    db: Database,
}

impl SqliteAttemptRepository {
    /// Crée le dépôt.
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

fn to_repository_error(error: DbError) -> RepositoryError {
    match error {
        DbError::Sqlite(rusqlite::Error::SqliteFailure(e, _))
            if e.code == ErrorCode::ConstraintViolation =>
        {
            // Clé étrangère : le profil n'existe plus.
            RepositoryError::NotFound
        }
        DbError::Closed => RepositoryError::Unavailable(DbError::Closed.to_string()),
        other => {
            tracing::error!(error = %other, "erreur de la base (tentatives)");
            RepositoryError::Backend(other.to_string())
        }
    }
}

impl AttemptRepository for SqliteAttemptRepository {
    fn record(
        &self,
        profile: ProfileId,
        exercise: ContentId,
        outcome: Outcome,
        hints_used: u8,
    ) -> BoxFuture<'_, RepoResult<()>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    conn.execute(
                        "INSERT INTO exercise_attempts
                             (profile_id, exercise_id, outcome, hints_used, answered_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            profile.get(),
                            exercise.as_str(),
                            outcome.as_str(),
                            hints_used,
                            Timestamp::now().as_millisecond()
                        ],
                    )?;
                    Ok(())
                })
                .await
                .map_err(to_repository_error)
        })
    }

    fn progress(&self, profile: ProfileId) -> BoxFuture<'_, RepoResult<ProgressMap>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let mut stmt = conn.prepare_cached(
                        "SELECT exercise_id, outcome, hints_used FROM exercise_attempts
                         WHERE profile_id = ?1 ORDER BY id",
                    )?;
                    let rows = stmt.query_map([profile.get()], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, i64>(2)?,
                        ))
                    })?;
                    let mut progress = ProgressMap::new();
                    for row in rows {
                        let (exercise, outcome, hints) = row?;
                        // Une ligne illisible (contenu retiré, valeur inconnue)
                        // est ignorée plutôt que de bloquer toute la progression.
                        let (Some(exercise), Some(outcome)) =
                            (ContentId::parse(&exercise), Outcome::parse(&outcome))
                        else {
                            tracing::warn!(%exercise, %outcome, "tentative illisible ignorée");
                            continue;
                        };
                        let hints = u8::try_from(hints).unwrap_or(u8::MAX);
                        progress.entry(exercise).or_default().record(outcome, hints);
                    }
                    Ok(progress)
                })
                .await
                .map_err(to_repository_error)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sqlite::open;
    use crate::sqlite::profiles::SqliteProfileRepository;
    use tssrlabs_application::ports::ProfileRepository;
    use tssrlabs_domain::exercise::ExerciseProgress;
    use tssrlabs_domain::profile::ProfileName;

    fn id(v: &str) -> ContentId {
        ContentId::parse(v).unwrap()
    }

    #[tokio::test]
    async fn attempts_build_progress_per_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let (db, _thread) = open(&tmp.path().join("db"), tmp.path()).unwrap();
        let profiles = SqliteProfileRepository::new(db.clone());
        let alice = profiles
            .insert(ProfileName::parse("Alice").unwrap())
            .await
            .unwrap();
        let bob = profiles
            .insert(ProfileName::parse("Bob").unwrap())
            .await
            .unwrap();
        let repo = SqliteAttemptRepository::new(db);

        repo.record(alice.id, id("ex-a"), Outcome::Incorrect, 0)
            .await
            .unwrap();
        repo.record(alice.id, id("ex-a"), Outcome::Correct, 1)
            .await
            .unwrap();
        repo.record(alice.id, id("ex-b"), Outcome::Revealed, 2)
            .await
            .unwrap();
        repo.record(bob.id, id("ex-a"), Outcome::Correct, 0)
            .await
            .unwrap();

        let progress = repo.progress(alice.id).await.unwrap();
        assert_eq!(
            progress[&id("ex-a")],
            ExerciseProgress {
                attempts: 2,
                solved: true,
                revealed: false,
                fewest_hints: Some(1)
            }
        );
        assert!(progress[&id("ex-b")].revealed);
        assert_eq!(repo.progress(bob.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn attempts_follow_profile_deletion_and_reject_unknown_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let (db, _thread) = open(&tmp.path().join("db"), tmp.path()).unwrap();
        let profiles = SqliteProfileRepository::new(db.clone());
        let alice = profiles
            .insert(ProfileName::parse("Alice").unwrap())
            .await
            .unwrap();
        let repo = SqliteAttemptRepository::new(db);
        repo.record(alice.id, id("ex-a"), Outcome::Correct, 0)
            .await
            .unwrap();

        profiles.delete(alice.id).await.unwrap();
        assert!(repo.progress(alice.id).await.unwrap().is_empty());
        assert_eq!(
            repo.record(alice.id, id("ex-a"), Outcome::Correct, 0).await,
            Err(RepositoryError::NotFound)
        );
    }
}
