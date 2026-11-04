//! Cas d'utilisation : profils locaux et préférences.

use std::path::PathBuf;
use std::sync::Arc;

use tssrlabs_domain::preferences::Preferences;
use tssrlabs_domain::profile::{
    Profile, ProfileId, ProfileName, ProfileNameError, sort_for_display,
};

use crate::ports::{ProfileRepository, RepositoryError, SafetyBackup};

/// Échec d'un cas d'utilisation sur les profils.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProfileError {
    /// Nom refusé par les règles du domaine.
    #[error("nom de profil invalide : {0}")]
    InvalidName(#[from] ProfileNameError),
    /// Nom déjà utilisé par un autre profil.
    #[error("ce nom de profil est déjà utilisé")]
    NameTaken,
    /// Profil introuvable (supprimé entre-temps).
    #[error("profil introuvable")]
    NotFound,
    /// La confirmation de suppression ne reproduit pas le nom du profil.
    #[error("la confirmation ne correspond pas au nom du profil")]
    ConfirmationMismatch,
    /// Erreur de stockage.
    #[error(transparent)]
    Storage(RepositoryError),
}

impl From<RepositoryError> for ProfileError {
    fn from(error: RepositoryError) -> Self {
        match error {
            RepositoryError::Conflict => Self::NameTaken,
            RepositoryError::NotFound => Self::NotFound,
            other => Self::Storage(other),
        }
    }
}

/// Profil actif et ses préférences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveProfile {
    /// Profil.
    pub profile: Profile,
    /// Préférences du profil.
    pub preferences: Preferences,
}

/// État complet des profils, renvoyé après chaque opération pour que
/// l'interface reflète toujours l'état réellement stocké.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfilesSnapshot {
    /// Profils, les plus récemment utilisés d'abord.
    pub profiles: Vec<Profile>,
    /// Profil actif, s'il y en a un.
    pub active: Option<ActiveProfile>,
}

/// Résultat d'une suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deletion {
    /// État après suppression.
    pub snapshot: ProfilesSnapshot,
    /// Copie de sécurité créée avant la suppression.
    pub backup: PathBuf,
}

/// Service des profils.
#[derive(Debug, Clone)]
pub struct ProfileService {
    repository: Arc<dyn ProfileRepository>,
    backup: Arc<dyn SafetyBackup>,
}

impl ProfileService {
    /// Crée le service.
    pub fn new(repository: Arc<dyn ProfileRepository>, backup: Arc<dyn SafetyBackup>) -> Self {
        Self { repository, backup }
    }

    /// État courant.
    pub async fn snapshot(&self) -> Result<ProfilesSnapshot, ProfileError> {
        let mut profiles = self.repository.list().await?;
        sort_for_display(&mut profiles);
        let active = match self.repository.active_profile().await? {
            Some(id) => match profiles.iter().find(|p| p.id == id) {
                Some(profile) => Some(ActiveProfile {
                    profile: profile.clone(),
                    preferences: self.repository.load_preferences(id).await?,
                }),
                None => None,
            },
            None => None,
        };
        Ok(ProfilesSnapshot { profiles, active })
    }

    /// Crée un profil et l'active.
    pub async fn create(&self, raw_name: &str) -> Result<ProfilesSnapshot, ProfileError> {
        let name = ProfileName::parse(raw_name)?;
        let profile = self.repository.insert(name).await?;
        tracing::info!(profile = profile.id.get(), "profil créé");
        self.repository.set_active_profile(profile.id).await?;
        self.snapshot().await
    }

    /// Renomme un profil.
    pub async fn rename(
        &self,
        id: ProfileId,
        raw_name: &str,
    ) -> Result<ProfilesSnapshot, ProfileError> {
        let name = ProfileName::parse(raw_name)?;
        self.repository.rename(id, name).await?;
        tracing::info!(profile = id.get(), "profil renommé");
        self.snapshot().await
    }

    /// Active un profil.
    pub async fn select(&self, id: ProfileId) -> Result<ProfilesSnapshot, ProfileError> {
        self.repository.set_active_profile(id).await?;
        tracing::info!(profile = id.get(), "profil activé");
        self.snapshot().await
    }

    /// Supprime un profil après confirmation par saisie de son nom.
    ///
    /// Une copie de sécurité des données est créée **avant** la suppression ;
    /// si elle échoue, rien n'est supprimé.
    pub async fn delete(
        &self,
        id: ProfileId,
        confirmation: &str,
    ) -> Result<Deletion, ProfileError> {
        let profiles = self.repository.list().await?;
        let profile = profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or(ProfileError::NotFound)?;
        if !profile.name.matches_confirmation(confirmation) {
            return Err(ProfileError::ConfirmationMismatch);
        }
        let backup = self.backup.backup("suppression-profil").await?;
        tracing::info!(profile = id.get(), backup = %backup.display(), "copie de sécurité créée");
        self.repository.delete(id).await?;
        tracing::info!(profile = id.get(), "profil supprimé");
        Ok(Deletion {
            snapshot: self.snapshot().await?,
            backup,
        })
    }

    /// Enregistre les préférences d'un profil.
    pub async fn save_preferences(
        &self,
        id: ProfileId,
        preferences: Preferences,
    ) -> Result<(), ProfileError> {
        self.repository.save_preferences(id, preferences).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::future::BoxFuture;
    use jiff::Timestamp;
    use std::sync::Mutex;
    use tssrlabs_domain::preferences::ThemeChoice;

    use crate::ports::RepoResult;

    /// Double de test en mémoire (jamais utilisé hors des tests).
    #[derive(Debug, Default)]
    struct Memory {
        state: Mutex<MemoryState>,
    }

    #[derive(Debug, Default)]
    struct MemoryState {
        next_id: i64,
        profiles: Vec<Profile>,
        active: Option<ProfileId>,
        preferences: Vec<(ProfileId, Preferences)>,
        backups: Vec<String>,
        fail_backup: bool,
        clock: i64,
    }

    impl MemoryState {
        fn tick(&mut self) -> Timestamp {
            self.clock += 1;
            Timestamp::from_second(self.clock).unwrap()
        }
    }

    impl ProfileRepository for Memory {
        fn list(&self) -> BoxFuture<'_, RepoResult<Vec<Profile>>> {
            Box::pin(async move { Ok(self.state.lock().unwrap().profiles.clone()) })
        }

        fn insert(&self, name: ProfileName) -> BoxFuture<'_, RepoResult<Profile>> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                if state.profiles.iter().any(|p| p.name.key() == name.key()) {
                    return Err(RepositoryError::Conflict);
                }
                state.next_id += 1;
                let profile = Profile {
                    id: ProfileId::new(state.next_id),
                    name,
                    created_at: state.tick(),
                    last_used_at: None,
                };
                state.profiles.push(profile.clone());
                Ok(profile)
            })
        }

        fn rename(&self, id: ProfileId, name: ProfileName) -> BoxFuture<'_, RepoResult<()>> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                if state
                    .profiles
                    .iter()
                    .any(|p| p.id != id && p.name.key() == name.key())
                {
                    return Err(RepositoryError::Conflict);
                }
                let profile = state
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == id)
                    .ok_or(RepositoryError::NotFound)?;
                profile.name = name;
                Ok(())
            })
        }

        fn delete(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<()>> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                let before = state.profiles.len();
                state.profiles.retain(|p| p.id != id);
                if state.profiles.len() == before {
                    return Err(RepositoryError::NotFound);
                }
                state.preferences.retain(|(p, _)| *p != id);
                if state.active == Some(id) {
                    state.active = None;
                }
                Ok(())
            })
        }

        fn active_profile(&self) -> BoxFuture<'_, RepoResult<Option<ProfileId>>> {
            Box::pin(async move { Ok(self.state.lock().unwrap().active) })
        }

        fn set_active_profile(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<()>> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                let now = state.tick();
                let profile = state
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == id)
                    .ok_or(RepositoryError::NotFound)?;
                profile.last_used_at = Some(now);
                state.active = Some(id);
                Ok(())
            })
        }

        fn load_preferences(&self, id: ProfileId) -> BoxFuture<'_, RepoResult<Preferences>> {
            Box::pin(async move {
                let state = self.state.lock().unwrap();
                Ok(state
                    .preferences
                    .iter()
                    .find(|(p, _)| *p == id)
                    .map(|(_, prefs)| *prefs)
                    .unwrap_or_default())
            })
        }

        fn save_preferences(
            &self,
            id: ProfileId,
            preferences: Preferences,
        ) -> BoxFuture<'_, RepoResult<()>> {
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                state.preferences.retain(|(p, _)| *p != id);
                state.preferences.push((id, preferences));
                Ok(())
            })
        }
    }

    impl SafetyBackup for Memory {
        fn backup(&self, reason: &str) -> BoxFuture<'_, RepoResult<PathBuf>> {
            let reason = reason.to_owned();
            Box::pin(async move {
                let mut state = self.state.lock().unwrap();
                if state.fail_backup {
                    return Err(RepositoryError::Backend("disque plein".into()));
                }
                state.backups.push(reason.clone());
                Ok(PathBuf::from(format!("sauvegarde-{reason}")))
            })
        }
    }

    fn service() -> (ProfileService, Arc<Memory>) {
        let memory = Arc::new(Memory::default());
        (ProfileService::new(memory.clone(), memory.clone()), memory)
    }

    #[tokio::test]
    async fn create_activates_the_new_profile() {
        let (service, _) = service();
        let snapshot = service.create("  Alice  ").await.unwrap();
        assert_eq!(snapshot.profiles.len(), 1);
        let active = snapshot.active.unwrap();
        assert_eq!(active.profile.name.as_str(), "Alice");
        assert_eq!(active.preferences, Preferences::default());
    }

    #[tokio::test]
    async fn duplicate_names_are_refused_case_insensitively() {
        let (service, _) = service();
        service.create("Alice").await.unwrap();
        assert_eq!(service.create("ALICE").await, Err(ProfileError::NameTaken));
    }

    #[tokio::test]
    async fn invalid_name_is_refused_before_storage() {
        let (service, memory) = service();
        assert_eq!(
            service.create("   ").await,
            Err(ProfileError::InvalidName(ProfileNameError::Empty))
        );
        assert!(memory.state.lock().unwrap().profiles.is_empty());
    }

    #[tokio::test]
    async fn select_switches_active_profile_and_orders_by_recent_use() {
        let (service, _) = service();
        let alice = service
            .create("Alice")
            .await
            .unwrap()
            .active
            .unwrap()
            .profile;
        service.create("Bob").await.unwrap();
        let snapshot = service.select(alice.id).await.unwrap();
        assert_eq!(snapshot.active.unwrap().profile.id, alice.id);
        assert_eq!(snapshot.profiles[0].id, alice.id);
    }

    #[tokio::test]
    async fn rename_conflict_and_success() {
        let (service, _) = service();
        let alice = service
            .create("Alice")
            .await
            .unwrap()
            .active
            .unwrap()
            .profile;
        service.create("Bob").await.unwrap();
        assert_eq!(
            service.rename(alice.id, "bob").await,
            Err(ProfileError::NameTaken)
        );
        let snapshot = service.rename(alice.id, "Alicia").await.unwrap();
        assert!(
            snapshot
                .profiles
                .iter()
                .any(|p| p.name.as_str() == "Alicia")
        );
    }

    #[tokio::test]
    async fn delete_requires_confirmation_and_backs_up_first() {
        let (service, memory) = service();
        let alice = service
            .create("Alice")
            .await
            .unwrap()
            .active
            .unwrap()
            .profile;

        assert_eq!(
            service.delete(alice.id, "alice").await,
            Err(ProfileError::ConfirmationMismatch)
        );
        assert!(memory.state.lock().unwrap().backups.is_empty());

        let deletion = service.delete(alice.id, "Alice").await.unwrap();
        assert!(deletion.snapshot.profiles.is_empty());
        assert_eq!(deletion.snapshot.active, None);
        assert_eq!(
            memory.state.lock().unwrap().backups,
            vec!["suppression-profil"]
        );
    }

    #[tokio::test]
    async fn failed_backup_prevents_deletion() {
        let (service, memory) = service();
        let alice = service
            .create("Alice")
            .await
            .unwrap()
            .active
            .unwrap()
            .profile;
        memory.state.lock().unwrap().fail_backup = true;

        assert!(matches!(
            service.delete(alice.id, "Alice").await,
            Err(ProfileError::Storage(_))
        ));
        assert_eq!(memory.state.lock().unwrap().profiles.len(), 1);
    }

    #[tokio::test]
    async fn preferences_are_saved_per_profile() {
        let (service, _) = service();
        let alice = service
            .create("Alice")
            .await
            .unwrap()
            .active
            .unwrap()
            .profile;
        let prefs = Preferences {
            theme: ThemeChoice::HighContrast,
        };
        service.save_preferences(alice.id, prefs).await.unwrap();
        service.create("Bob").await.unwrap();
        let snapshot = service.select(alice.id).await.unwrap();
        assert_eq!(snapshot.active.unwrap().preferences, prefs);
    }
}
