//! Point d'entrée de TSSR Labs : racine de composition.
//!
//! Ce binaire assemble les couches (infrastructure → présentation), possède
//! le runtime Tokio et garantit un arrêt ordonné : annulation des tâches,
//! restauration du terminal, vidage des journaux.

mod cli;

use std::io::IsTerminal;
use std::process::ExitCode;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tssrlabs_application::Services;
use tssrlabs_application::profiles::ProfileService;
use tssrlabs_infrastructure::host;
use tssrlabs_infrastructure::logging::{self, LoggingError};
use tssrlabs_infrastructure::paths::{AppPaths, PathsError};
use tssrlabs_infrastructure::sqlite::profiles::{SqliteProfileRepository, SqliteSafetyBackup};
use tssrlabs_infrastructure::sqlite::{self, DbError};
use tssrlabs_tui::TuiError;
use tssrlabs_tui::app::{App, Environment};
use tssrlabs_tui::terminal::TerminalSession;

use crate::cli::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Code de sortie pour une erreur d'utilisation de la ligne de commande.
const EXIT_USAGE: u8 = 2;

/// Erreurs de haut niveau, présentées à l'utilisateur en français.
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("TSSR Labs doit être lancé dans un terminal interactif")]
    NotATerminal,
    #[error(transparent)]
    Paths(#[from] PathsError),
    #[error(transparent)]
    Logging(#[from] LoggingError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error("impossible de démarrer le moteur asynchrone")]
    Runtime(#[source] std::io::Error),
    #[error("impossible de préparer le terminal")]
    TerminalSetup(#[source] std::io::Error),
    #[error(transparent)]
    Tui(#[from] TuiError),
}

fn main() -> ExitCode {
    let command = match cli::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(err) => {
            eprintln!("tssrlabs : {err}\n\n{}", cli::HELP);
            return ExitCode::from(EXIT_USAGE);
        }
    };

    let result = match command {
        Command::Help => {
            print!("{}", cli::HELP);
            Ok(())
        }
        Command::Version => {
            println!("tssrlabs {VERSION}");
            Ok(())
        }
        Command::Paths => print_paths(),
        Command::Run => run(),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Erreur : {}", error_chain(&err));
            ExitCode::FAILURE
        }
    }
}

fn print_paths() -> Result<(), AppError> {
    let paths = AppPaths::discover()?;
    println!("Données       : {}", paths.data_dir().display());
    println!("Configuration : {}", paths.config_dir().display());
    println!("Journaux      : {}", paths.log_dir().display());
    Ok(())
}

fn run() -> Result<(), AppError> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(AppError::NotATerminal);
    }

    let paths = AppPaths::discover()?;
    paths.ensure_dirs()?;
    let _log_guard = logging::init(paths.log_dir())?;
    let host = host::detect();
    tracing::info!(
        version = VERSION,
        os = %host.os,
        arch = %host.arch,
        term = ?host.terminal.term,
        terminal = ?host.terminal.program,
        colors = ?host.terminal.color,
        colors_source = ?host.terminal.color_source,
        "démarrage de TSSR Labs"
    );
    install_panic_logging();
    let env = Environment {
        version: VERSION,
        host,
        data_dir: paths.data_dir().to_path_buf(),
        config_dir: paths.config_dir().to_path_buf(),
        log_dir: paths.log_dir().to_path_buf(),
        database_file: paths.database_file(),
    };

    // Ouverture synchrone avant l'interface : une base illisible ou plus
    // récente doit être signalée clairement, sans rien modifier.
    let (db, db_thread) = sqlite::open(&paths.database_file(), &paths.safety_backup_dir())?;
    let services = Services {
        profiles: ProfileService::new(
            Arc::new(SqliteProfileRepository::new(db.clone())),
            Arc::new(SqliteSafetyBackup::new(db)),
        ),
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("tssrlabs-worker")
        .build()
        .map_err(AppError::Runtime)?;
    let result = runtime.block_on(run_interface(env, services));
    // Aucune tâche ne doit survivre : `run_interface` a déjà attendu les
    // siennes ; on borne néanmoins l'arrêt du runtime.
    runtime.shutdown_timeout(std::time::Duration::from_secs(2));
    // Les services sont détruits avec `run_interface` ; on ferme ensuite la
    // base après ses dernières écritures.
    db_thread.close();

    match &result {
        Ok(()) => tracing::info!("arrêt normal"),
        Err(err) => tracing::error!(error = %error_chain(err), "arrêt sur erreur"),
    }
    result
}

/// Lance l'interface et orchestre l'arrêt ordonné des tâches de fond.
async fn run_interface(env: Environment, services: Services) -> Result<(), AppError> {
    let shutdown = CancellationToken::new();
    let tasks = TaskTracker::new();
    tasks.spawn(cancel_on_signal(shutdown.clone()));

    let result = {
        let mut session = TerminalSession::enter().map_err(AppError::TerminalSetup)?;
        let mut app = App::new(env);
        tssrlabs_tui::run::run(session.terminal_mut(), &mut app, services, &shutdown).await
        // `session` est détruite ici : le terminal est restauré avant tout
        // affichage d'erreur sur la sortie standard.
    };

    shutdown.cancel();
    tasks.close();
    tasks.wait().await;
    Ok(result?)
}

/// Annule `token` à la réception d'un signal d'arrêt du système, ou se
/// termine dès que `token` est annulé par ailleurs.
async fn cancel_on_signal(token: CancellationToken) {
    tokio::select! {
        () = token.cancelled() => {}
        () = shutdown_signal() => token.cancel(),
    }
}

#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let (Ok(mut interrupt), Ok(mut terminate), Ok(mut hangup)) = (
        signal(SignalKind::interrupt()),
        signal(SignalKind::terminate()),
        signal(SignalKind::hangup()),
    ) else {
        tracing::warn!("écoute des signaux système impossible");
        return std::future::pending().await;
    };
    tokio::select! {
        _ = interrupt.recv() => {}
        _ = terminate.recv() => {}
        _ = hangup.recv() => {}
    }
}

#[cfg(windows)]
async fn shutdown_signal() {
    use tokio::signal::windows::{ctrl_c, ctrl_close};

    let (Ok(mut interrupt), Ok(mut close)) = (ctrl_c(), ctrl_close()) else {
        tracing::warn!("écoute des signaux système impossible");
        return std::future::pending().await;
    };
    tokio::select! {
        _ = interrupt.recv() => {}
        _ = close.recv() => {}
    }
}

#[cfg(not(any(unix, windows)))]
async fn shutdown_signal() {
    std::future::pending().await
}

/// Journalise les paniques avant le crochet précédent. Installé avant la
/// préparation du terminal, il est appelé après la restauration effectuée
/// par le crochet de Ratatui.
fn install_panic_logging() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(panic = %info, "panique");
        previous(info);
    }));
}

/// Concatène une erreur et ses causes : « a : b : c ».
fn error_chain(err: &dyn std::error::Error) -> String {
    let mut text = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        text.push_str(" : ");
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}
