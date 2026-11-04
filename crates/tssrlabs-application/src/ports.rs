//! Ports : ce dont la couche application a besoin de l'infrastructure.
//!
//! Les méthodes renvoient des [`BoxFuture`] afin que les ports restent
//! utilisables derrière `Arc<dyn …>` (choix de l'implémentation à
//! l'exécution, doubles de test).

use std::path::PathBuf;

use futures_util::future::BoxFuture;
use tssrlabs_domain::preferences::Preferences;
use tssrlabs_domain::profile::{Profile, ProfileId, ProfileName};

/// Erreur d'un adaptateur de stockage.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RepositoryError {
    /// Contrainte d'unicité violée (nom déjà utilisé).
    #[error("conflit d'unicité")]
    Conflict,
    /// Élément introuvable.
    #[error("élément introuvable")]
    NotFound,
    /// Stockage indisponible (fermé, verrouillé…).
    #[error("stockage indisponible : {0}")]
    Unavailable(String),
    /// Autre erreur du stockage ; le message ne contient aucun secret.
    #[error("erreur du stockage : {0}")]
    Backend(String),
}

/// Résultat d'une opération de stockage.
pub type RepoResult<T> = Result<T, RepositoryError>;

/// Persistance des profils, du profil actif et des préférences.
pub trait ProfileRepository: Send + Sync + std::fmt::Debug {
    /// Tous les profils.
    fn list(&self) -> BoxFuture<'_, RepoResult<Vec<Profile>>>;
    /// Crée un profil ; [`RepositoryError::Conflict`] si le nom existe déjà
    /// (sans tenir compte de la casse).
    fn insert(&self, name: ProfileName) -> BoxFuture<'_, RepoResult<Profile>>;
    /// Renomme un profil.
    fn rename(&self, id: ProfileId, name: ProfileName) -> BoxFuture<'_, RepoResult<()>>;
    /// Supprime un profil et ses données ; s'il était actif, il ne l'est plus.
    fn delete(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<()>>;
    /// Profil actif mémorisé, s'il existe encore.
    fn active_profile(&self) -> BoxFuture<'_, RepoResult<Option<ProfileId>>>;
    /// Mémorise le profil actif et met à jour sa date de dernière utilisation.
    fn set_active_profile(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<()>>;
    /// Préférences d'un profil (valeurs par défaut si absentes).
    fn load_preferences(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<Preferences>>;
    /// Enregistre les préférences d'un profil.
    fn save_preferences(
        &self,
        id: ProfileId,
        preferences: Preferences,
    ) -> BoxFuture<'_, RepoResult<()>>;
}

/// Copie de sécurité avant une opération destructive.
pub trait SafetyBackup: Send + Sync + std::fmt::Debug {
    /// Crée une copie cohérente des données et renvoie son emplacement.
    /// `reason` est une étiquette courte intégrée au nom du fichier.
    fn backup(&self, reason: &str) -> BoxFuture<'_, RepoResult<PathBuf>>;
}
