//! Implémentation SQLite des tentatives d'exercices.

use futures_util::future::BoxFuture;
use jiff::Timestamp;
use rusqlite::{ErrorCode, params};
use tssrlabs_application::ports::{AttemptRepository, RepoResult, RepositoryError};
use tssrlabs_domain::content::ContentId;
use tssrlabs_domain::exercise::Outcome;
use tssrlabs_domain::profile::ProfileId;
use tssrlabs_domain::progression::AttemptRecord;

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
    ) -> BoxFuture<'_, RepoResult<AttemptRecord>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let at = Timestamp::now();
                    conn.execute(
                        "INSERT INTO exercise_attempts
                             (profile_id, exercise_id, outcome, hints_used, answered_at)
                         VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![
                            profile.get(),
                            exercise.as_str(),
                            outcome.as_str(),
                            hints_used,
                            at.as_millisecond()
                        ],
                    )?;
                    Ok(AttemptRecord {
                        exercise,
                        outcome,
                        hints_used,
                        // Même précision que la valeur stockée.
                        at: Timestamp::from_millisecond(at.as_millisecond()).unwrap_or(at),
                    })
                })
                .await
                .map_err(to_repository_error)
        })
    }

    fn attempts(&self, profile: ProfileId) -> BoxFuture<'_, RepoResult<Vec<AttemptRecord>>> {
        Box::pin(async move {
            self.db
                .call(move |conn| {
                    let mut stmt = conn.prepare_cached(
                        "SELECT exercise_id, outcome, hints_used, answered_at FROM exercise_attempts
                         WHERE profile_id = ?1 ORDER BY id",
                    )?;
                    let rows = stmt.query_map([profile.get()], |row| {
                        Ok((
                            row.get::<_, String>(0)?,
                            row.get::<_, String>(1)?,
                            row.get::<_, i64>(2)?,
                            row.get::<_, i64>(3)?,
                        ))
                    })?;
                    let mut attempts = Vec::new();
                    for row in rows {
                        let (exercise, outcome, hints, at) = row?;
                        // Une ligne illisible (valeur inconnue, date invalide)
                        // est ignorée plutôt que de bloquer toute la progression.
                        let (Some(exercise), Some(outcome), Ok(at)) = (
                            ContentId::parse(&exercise),
                            Outcome::parse(&outcome),
                            Timestamp::from_millisecond(at),
                        ) else {
                            tracing::warn!(%exercise, %outcome, "tentative illisible ignorée");
                            continue;
                        };
                        attempts.push(AttemptRecord {
                            exercise,
                            outcome,
                            hints_used: u8::try_from(hints).unwrap_or(u8::MAX),
                            at,
                        });
                    }
                    Ok(attempts)
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
    use tssrlabs_domain::profile::ProfileName;

    fn id(v: &str) -> ContentId {
        ContentId::parse(v).unwrap()
    }

    #[tokio::test]
    async fn attempts_are_returned_in_order_per_profile() {
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

        let first = repo
            .record(alice.id, id("ex-a"), Outcome::Incorrect, 0)
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

        let attempts = repo.attempts(alice.id).await.unwrap();
        assert_eq!(attempts.len(), 3);
        assert_eq!(attempts[0], first, "date relue identique à celle renvoyée");
        let outcomes: Vec<_> = attempts
            .iter()
            .map(|a| (a.exercise.as_str(), a.outcome, a.hints_used))
            .collect();
        assert_eq!(
            outcomes,
            vec![
                ("ex-a", Outcome::Incorrect, 0),
                ("ex-a", Outcome::Correct, 1),
                ("ex-b", Outcome::Revealed, 2)
            ]
        );
        assert_eq!(repo.attempts(bob.id).await.unwrap().len(), 1);
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
        assert!(repo.attempts(alice.id).await.unwrap().is_empty());
        assert_eq!(
            repo.record(alice.id, id("ex-a"), Outcome::Correct, 0)
                .await
                .map(|_| ()),
            Err(RepositoryError::NotFound)
        );
    }
}
