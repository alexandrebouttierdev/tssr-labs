//! Journaux locaux de TSSR Labs.
//!
//! - Fichiers uniquement, dans le répertoire de journaux de l'utilisateur ;
//!   rien n'est envoyé sur le réseau (aucune télémétrie).
//! - Rotation quotidienne, conservation limitée à [`MAX_LOG_FILES`] fichiers.
//! - Chaque ligne passe par [`crate::redact::redact`] avant d'être écrite.
//! - Écriture non bloquante : un thread dédié écrit sur le disque ; le
//!   [`LoggingGuard`] doit rester vivant jusqu'à la fin du programme pour que
//!   les dernières lignes soient vidées.
//! - Niveau réglable par la variable [`LOG_ENV_VAR`] (syntaxe `EnvFilter`,
//!   par exemple `debug` ou `tssrlabs=debug,info`). Défaut : `info`.

use std::io::{self, Write};
use std::path::Path;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::MakeWriter;

use crate::redact::redact;

/// Variable d'environnement réglant le niveau de journalisation.
pub const LOG_ENV_VAR: &str = "TSSRLABS_LOG";
/// Préfixe des fichiers de journaux.
pub const LOG_FILE_PREFIX: &str = "tssrlabs";
/// Nombre maximal de fichiers journaliers conservés.
pub const MAX_LOG_FILES: usize = 14;

/// Erreurs d'initialisation des journaux.
#[derive(Debug, thiserror::Error)]
pub enum LoggingError {
    /// Le fichier de journal ne peut pas être ouvert.
    #[error("impossible d'ouvrir les journaux dans {path}")]
    Appender {
        /// Répertoire des journaux.
        path: String,
        /// Erreur d'origine.
        #[source]
        source: tracing_appender::rolling::InitError,
    },
    /// Un collecteur `tracing` global est déjà installé.
    #[error("la journalisation est déjà initialisée")]
    AlreadyInitialized,
}

/// Garde de la journalisation : vide les journaux en attente quand il est détruit.
#[must_use = "les journaux sont perdus si le garde est détruit immédiatement"]
#[derive(Debug)]
pub struct LoggingGuard {
    _worker: WorkerGuard,
}

/// Installe le collecteur global écrivant dans `log_dir`.
///
/// `log_dir` doit exister (voir [`crate::paths::AppPaths::ensure_dirs`]).
pub fn init(log_dir: &Path) -> Result<LoggingGuard, LoggingError> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix("log")
        .max_log_files(MAX_LOG_FILES)
        .build(log_dir)
        .map_err(|source| LoggingError::Appender {
            path: log_dir.display().to_string(),
            source,
        })?;
    let (writer, worker) = tracing_appender::non_blocking(appender);

    // Une valeur invalide dans la variable d'environnement ne doit pas
    // empêcher le démarrage : on retombe sur le niveau par défaut.
    let filter = EnvFilter::try_from_env(LOG_ENV_VAR).unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_target(true)
        .with_writer(RedactingMakeWriter::new(writer))
        .try_init()
        .map_err(|_| LoggingError::AlreadyInitialized)?;

    Ok(LoggingGuard { _worker: worker })
}

/// Enveloppe un [`MakeWriter`] pour masquer les secrets de chaque écriture.
#[derive(Debug, Clone)]
pub struct RedactingMakeWriter<M> {
    inner: M,
}

impl<M> RedactingMakeWriter<M> {
    /// Crée l'enveloppe.
    pub fn new(inner: M) -> Self {
        Self { inner }
    }
}

impl<'a, M: MakeWriter<'a>> MakeWriter<'a> for RedactingMakeWriter<M> {
    type Writer = RedactingWriter<M::Writer>;

    fn make_writer(&'a self) -> Self::Writer {
        RedactingWriter {
            inner: self.inner.make_writer(),
        }
    }
}

/// Écrivain qui masque les secrets avant de transmettre les octets.
///
/// `tracing-subscriber` formate chaque événement complet dans un tampon puis
/// l'écrit en un seul appel : le masquage s'applique donc à la ligne entière.
#[derive(Debug)]
pub struct RedactingWriter<W> {
    inner: W,
}

impl<W: Write> Write for RedactingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let text = String::from_utf8_lossy(buf);
        self.inner.write_all(redact(&text).as_bytes())?;
        // Toute l'entrée a été consommée, même si la sortie a une autre taille.
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacting_writer_masks_secrets_and_reports_full_consumption() {
        let mut out = Vec::new();
        let input = b"connexion avec api_key=abcdef123 au fournisseur\n";
        let written = RedactingWriter { inner: &mut out }.write(input).unwrap();

        assert_eq!(written, input.len());
        let text = String::from_utf8(out).unwrap();
        assert!(!text.contains("abcdef123"), "{text}");
        assert!(text.contains("au fournisseur"), "{text}");
    }

    #[test]
    fn subscriber_output_goes_through_redaction() {
        use std::sync::{Arc, Mutex};

        #[derive(Clone, Default)]
        struct Shared(Arc<Mutex<Vec<u8>>>);
        impl Write for Shared {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let sink = Shared::default();
        let make = {
            let sink = sink.clone();
            move || sink.clone()
        };
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_writer(RedactingMakeWriter::new(make))
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!(url = "https://bob:pa55@example.org/x", "requête");
        });

        let text = String::from_utf8(sink.0.lock().unwrap().clone()).unwrap();
        assert!(text.contains("requête"), "{text}");
        assert!(!text.contains("pa55"), "{text}");
    }

    #[test]
    fn init_fails_cleanly_when_log_dir_is_a_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("pas-un-dossier");
        std::fs::write(&file, b"").unwrap();

        assert!(matches!(init(&file), Err(LoggingError::Appender { .. })));
    }
}
