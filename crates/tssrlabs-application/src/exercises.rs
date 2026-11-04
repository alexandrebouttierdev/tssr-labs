//! Cas d'utilisation : tentatives d'exercices et progression.
//!
//! La vérification d'une réponse est une règle du domaine
//! ([`tssrlabs_domain::exercise::Exercise::check`]) ; ce service enregistre
//! les tentatives d'un profil et restitue sa progression.

use std::collections::HashMap;
use std::sync::Arc;

use tssrlabs_domain::content::ContentId;
use tssrlabs_domain::exercise::{ExerciseProgress, Outcome};
use tssrlabs_domain::profile::ProfileId;

use crate::ports::{AttemptRepository, RepositoryError};

/// Progression d'un profil, par exercice.
pub type ProgressMap = HashMap<ContentId, ExerciseProgress>;

/// Service des exercices.
#[derive(Debug, Clone)]
pub struct ExerciseService {
    attempts: Arc<dyn AttemptRepository>,
}

impl ExerciseService {
    /// Crée le service.
    pub fn new(attempts: Arc<dyn AttemptRepository>) -> Self {
        Self { attempts }
    }

    /// Enregistre une tentative et renvoie la progression à jour de
    /// l'exercice.
    pub async fn record(
        &self,
        profile: ProfileId,
        exercise: ContentId,
        outcome: Outcome,
        hints_used: u8,
    ) -> Result<(ContentId, ExerciseProgress), RepositoryError> {
        self.attempts
            .record(profile, exercise.clone(), outcome, hints_used)
            .await?;
        let progress = self
            .attempts
            .progress(profile)
            .await?
            .remove(&exercise)
            .unwrap_or_default();
        Ok((exercise, progress))
    }

    /// Progression complète d'un profil.
    pub async fn progress(&self, profile: ProfileId) -> Result<ProgressMap, RepositoryError> {
        self.attempts.progress(profile).await
    }
}
