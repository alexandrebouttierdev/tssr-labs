//! Cas d'utilisation : tentatives d'exercices.
//!
//! La vérification d'une réponse et le calcul de la progression (XP,
//! niveaux, maîtrise) sont des règles du domaine
//! ([`tssrlabs_domain::exercise`], [`tssrlabs_domain::progression`]) ; ce
//! service enregistre les tentatives d'un profil et les restitue.

use std::sync::Arc;

use tssrlabs_domain::content::ContentId;
use tssrlabs_domain::exercise::Outcome;
use tssrlabs_domain::profile::ProfileId;
use tssrlabs_domain::progression::AttemptRecord;

use crate::ports::{AttemptRepository, RepositoryError};

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

    /// Enregistre une tentative et la renvoie datée.
    pub async fn record(
        &self,
        profile: ProfileId,
        exercise: ContentId,
        outcome: Outcome,
        hints_used: u8,
    ) -> Result<AttemptRecord, RepositoryError> {
        self.attempts
            .record(profile, exercise, outcome, hints_used)
            .await
    }

    /// Tentatives d'un profil, dans l'ordre chronologique.
    pub async fn attempts(
        &self,
        profile: ProfileId,
    ) -> Result<Vec<AttemptRecord>, RepositoryError> {
        self.attempts.attempts(profile).await
    }
}
