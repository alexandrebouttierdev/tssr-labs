//! Boucle d'événements asynchrone de l'interface.

use crossterm::event::EventStream;
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio_util::sync::CancellationToken;

use crate::TuiError;
use crate::app::App;
use crate::ui;

/// Exécute la boucle jusqu'à ce que l'application demande à quitter, que
/// `shutdown` soit annulé (signal système) ou que le flux d'événements se
/// termine.
///
/// La boucle n'effectue que du dessin et des mises à jour d'état ; tout
/// traitement long sera délégué à des tâches dont les résultats reviendront
/// par canal, afin de ne jamais figer l'affichage.
pub async fn run(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    shutdown: &CancellationToken,
) -> Result<(), TuiError> {
    let mut events = EventStream::new();
    while app.is_running() {
        terminal.draw(|frame| ui::render(app, frame))?;
        tokio::select! {
            () = shutdown.cancelled() => {
                tracing::info!("arrêt demandé par un signal système");
                break;
            }
            event = events.next() => match event {
                Some(Ok(event)) => {
                    if let Some(action) = app.handle_event(&event) {
                        tracing::debug!(?action, "action");
                        app.update(action);
                    }
                }
                Some(Err(err)) => return Err(err.into()),
                None => break,
            },
        }
    }
    Ok(())
}
