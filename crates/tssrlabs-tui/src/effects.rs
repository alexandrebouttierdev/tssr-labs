//! Effets demandés par l'interface et résultats renvoyés.
//!
//! L'état ([`crate::app::App`]) ne réalise jamais d'entrée/sortie : il
//! produit des [`Effect`], exécutés hors de la boucle de rendu par un
//! **unique** travailleur séquentiel (l'ordre des demandes est donc
//! conservé). Chaque résultat revient sous forme d'[`AppEvent`].

use tokio::sync::mpsc;
use tssrlabs_application::Services;
use tssrlabs_application::ports::RepositoryError;
use tssrlabs_application::profiles::{Deletion, ProfileError, ProfilesSnapshot};
use tssrlabs_domain::content::ContentId;
use tssrlabs_domain::exercise::Outcome;
use tssrlabs_domain::preferences::Preferences;
use tssrlabs_domain::profile::ProfileId;
use tssrlabs_domain::progression::AttemptRecord;

/// Opération demandée à la couche application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Charger les profils et le profil actif.
    LoadProfiles,
    /// Créer et activer un profil.
    CreateProfile {
        /// Nom saisi.
        name: String,
    },
    /// Renommer un profil.
    RenameProfile {
        /// Profil concerné.
        id: ProfileId,
        /// Nouveau nom saisi.
        name: String,
    },
    /// Activer un profil.
    SelectProfile(ProfileId),
    /// Supprimer un profil (avec confirmation saisie).
    DeleteProfile {
        /// Profil concerné.
        id: ProfileId,
        /// Saisie de confirmation.
        confirmation: String,
    },
    /// Enregistrer les préférences d'un profil.
    SavePreferences {
        /// Profil concerné.
        id: ProfileId,
        /// Préférences.
        preferences: Preferences,
    },
    /// Charger la progression des exercices d'un profil.
    LoadProgress(ProfileId),
    /// Enregistrer une tentative d'exercice.
    RecordAttempt {
        /// Profil.
        profile: ProfileId,
        /// Exercice.
        exercise: ContentId,
        /// Issue.
        outcome: Outcome,
        /// Indices utilisés.
        hints_used: u8,
    },
}

/// Nature de l'opération ayant produit un état des profils.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileOp {
    /// Chargement initial.
    Load,
    /// Création.
    Create,
    /// Renommage.
    Rename(ProfileId),
    /// Activation.
    Select,
}

/// Résultat d'un effet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// État des profils après une opération.
    Profiles {
        /// Opération.
        op: ProfileOp,
        /// Résultat.
        result: Result<ProfilesSnapshot, ProfileError>,
    },
    /// Résultat d'une suppression.
    Deleted(Result<Deletion, ProfileError>),
    /// Résultat d'un enregistrement de préférences.
    PreferencesSaved(Result<(), ProfileError>),
    /// Progression chargée pour un profil.
    ProgressLoaded {
        /// Profil concerné (ignoré s'il n'est plus actif).
        profile: ProfileId,
        /// Tentatives, dans l'ordre chronologique.
        result: Result<Vec<AttemptRecord>, RepositoryError>,
    },
    /// Tentative enregistrée.
    AttemptRecorded {
        /// Profil concerné.
        profile: ProfileId,
        /// Tentative datée.
        result: Result<AttemptRecord, RepositoryError>,
    },
}

/// Exécute un effet.
pub async fn execute(effect: Effect, services: &Services) -> AppEvent {
    let profiles = &services.profiles;
    match effect {
        Effect::LoadProfiles => AppEvent::Profiles {
            op: ProfileOp::Load,
            result: profiles.snapshot().await,
        },
        Effect::CreateProfile { name } => AppEvent::Profiles {
            op: ProfileOp::Create,
            result: profiles.create(&name).await,
        },
        Effect::RenameProfile { id, name } => AppEvent::Profiles {
            op: ProfileOp::Rename(id),
            result: profiles.rename(id, &name).await,
        },
        Effect::SelectProfile(id) => AppEvent::Profiles {
            op: ProfileOp::Select,
            result: profiles.select(id).await,
        },
        Effect::DeleteProfile { id, confirmation } => {
            AppEvent::Deleted(profiles.delete(id, &confirmation).await)
        }
        Effect::SavePreferences { id, preferences } => {
            AppEvent::PreferencesSaved(profiles.save_preferences(id, preferences).await)
        }
        Effect::LoadProgress(profile) => AppEvent::ProgressLoaded {
            profile,
            result: services.exercises.attempts(profile).await,
        },
        Effect::RecordAttempt {
            profile,
            exercise,
            outcome,
            hints_used,
        } => AppEvent::AttemptRecorded {
            profile,
            result: services
                .exercises
                .record(profile, exercise, outcome, hints_used)
                .await,
        },
    }
}

/// Travailleur séquentiel : exécute les effets dans l'ordre de réception
/// jusqu'à la fermeture du canal d'entrée, puis se termine.
pub async fn worker(
    services: Services,
    mut effects: mpsc::UnboundedReceiver<Effect>,
    events: mpsc::UnboundedSender<AppEvent>,
) {
    while let Some(effect) = effects.recv().await {
        tracing::debug!(?effect, "exécution d'un effet");
        let event = execute(effect, &services).await;
        // L'interface peut déjà être fermée : le résultat est alors ignoré,
        // mais l'opération elle-même a bien été menée à terme.
        let _ = events.send(event);
    }
}
