//! Emplacements des données locales de TSSR Labs.
//!
//! Par défaut, les répertoires standards de l'OS sont utilisés
//! (XDG sous Linux, `~/Library` sous macOS, `%LOCALAPPDATA%`/`%APPDATA%`
//! sous Windows). La variable d'environnement [`HOME_ENV_VAR`] permet de
//! tout regrouper sous une racine unique (installation portable, tests).

use std::fs::DirBuilder;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

/// Variable d'environnement permettant de forcer une racine unique.
pub const HOME_ENV_VAR: &str = "TSSRLABS_HOME";

/// Erreurs liées à la résolution ou à la création des répertoires.
#[derive(Debug, thiserror::Error)]
pub enum PathsError {
    /// L'OS ne fournit pas de répertoire personnel exploitable.
    #[error(
        "impossible de déterminer les répertoires de l'utilisateur ; \
         définissez la variable {HOME_ENV_VAR}"
    )]
    NoHomeDirectory,
    /// Création d'un répertoire impossible.
    #[error("impossible de créer le répertoire {path}")]
    CreateDir {
        /// Répertoire concerné.
        path: PathBuf,
        /// Erreur d'origine.
        #[source]
        source: io::Error,
    },
}

/// Ensemble des répertoires utilisés par l'application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    data_dir: PathBuf,
    config_dir: PathBuf,
    log_dir: PathBuf,
}

impl AppPaths {
    /// Résout les répertoires : [`HOME_ENV_VAR`] s'il est défini et non vide,
    /// sinon les répertoires standards de l'OS. Ne crée rien sur le disque.
    pub fn discover() -> Result<Self, PathsError> {
        match std::env::var_os(HOME_ENV_VAR) {
            Some(root) if !root.is_empty() => Ok(Self::from_root(Path::new(&root))),
            _ => Self::from_os_defaults(),
        }
    }

    /// Regroupe tous les répertoires sous une racine unique.
    pub fn from_root(root: &Path) -> Self {
        Self {
            data_dir: root.join("data"),
            config_dir: root.join("config"),
            log_dir: root.join("logs"),
        }
    }

    fn from_os_defaults() -> Result<Self, PathsError> {
        let dirs = ProjectDirs::from("", "", "TSSR Labs").ok_or(PathsError::NoHomeDirectory)?;
        // Les données locales (labos, sauvegardes, journaux) ne doivent pas
        // être synchronisées par un profil itinérant Windows : on utilise
        // `data_local_dir`.
        let data_dir = dirs.data_local_dir().to_path_buf();
        Ok(Self {
            log_dir: data_dir.join("logs"),
            data_dir,
            config_dir: dirs.config_dir().to_path_buf(),
        })
    }

    /// Répertoire des données applicatives (SQLite, stockage des labos…).
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Répertoire de configuration.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Répertoire des journaux locaux.
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    /// Crée les répertoires manquants. Sous Unix, ils sont créés avec les
    /// permissions `0700` : journaux et données ne concernent que
    /// l'utilisateur courant. Les répertoires existants ne sont pas modifiés.
    pub fn ensure_dirs(&self) -> Result<(), PathsError> {
        for dir in [&self.data_dir, &self.config_dir, &self.log_dir] {
            create_private_dir(dir).map_err(|source| PathsError::CreateDir {
                path: dir.clone(),
                source,
            })?;
        }
        Ok(())
    }
}

fn create_private_dir(path: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_root_groups_everything_under_root() {
        let root = Path::new("racine");
        let paths = AppPaths::from_root(root);
        assert_eq!(paths.data_dir(), root.join("data"));
        assert_eq!(paths.config_dir(), root.join("config"));
        assert_eq!(paths.log_dir(), root.join("logs"));
    }

    #[test]
    fn ensure_dirs_creates_missing_directories_and_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = AppPaths::from_root(&tmp.path().join("tssrlabs"));

        paths.ensure_dirs().unwrap();
        paths.ensure_dirs().unwrap();

        assert!(paths.data_dir().is_dir());
        assert!(paths.config_dir().is_dir());
        assert!(paths.log_dir().is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn ensure_dirs_restricts_permissions_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let paths = AppPaths::from_root(tmp.path());
        paths.ensure_dirs().unwrap();

        let mode = std::fs::metadata(paths.log_dir())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o700);
    }

    #[test]
    fn ensure_dirs_reports_the_failing_path() {
        let tmp = tempfile::tempdir().unwrap();
        // Un fichier ordinaire empêche la création du répertoire.
        let blocker = tmp.path().join("bloque");
        std::fs::write(&blocker, b"").unwrap();
        let paths = AppPaths::from_root(&blocker);

        let err = paths.ensure_dirs().unwrap_err();
        assert!(matches!(err, PathsError::CreateDir { .. }));
    }
}
