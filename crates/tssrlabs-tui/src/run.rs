//! Boucle d'événements asynchrone de l'interface.

use crossterm::event::EventStream;
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tssrlabs_application::Services;

use crate::TuiError;
use crate::app::App;
use crate::effects;
use crate::ui;

/// Exécute la boucle jusqu'à ce que l'application demande à quitter, que
/// `shutdown` soit annulé (signal système) ou que le flux d'événements se
/// termine.
///
/// Les effets demandés par l'état sont confiés à un travailleur séquentiel
/// possédé par cette fonction : avant de rendre la main, elle attend la fin
/// des opérations déjà demandées (ex. enregistrement d'une préférence),
/// afin qu'aucune tâche ne survive à l'interface.
pub async fn run(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    services: Services,
    shutdown: &CancellationToken,
) -> Result<(), TuiError> {
    let tasks = TaskTracker::new();
    let (effect_tx, effect_rx) = mpsc::unbounded_channel();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel();
    tasks.spawn(effects::worker(services, effect_rx, event_tx));

    let result = event_loop(terminal, app, &effect_tx, &mut event_rx, shutdown).await;

    // Fermer le canal termine le travailleur après les effets en attente.
    drop(effect_tx);
    tasks.close();
    tasks.wait().await;
    result
}

async fn event_loop(
    terminal: &mut DefaultTerminal,
    app: &mut App,
    effect_tx: &mpsc::UnboundedSender<effects::Effect>,
    event_rx: &mut mpsc::UnboundedReceiver<effects::AppEvent>,
    shutdown: &CancellationToken,
) -> Result<(), TuiError> {
    let mut terminal_events = EventStream::new();
    loop {
        for effect in app.take_effects() {
            if effect_tx.send(effect).is_err() {
                tracing::error!("travailleur des effets arrêté");
            }
        }
        if !app.is_running() {
            return Ok(());
        }
        terminal.draw(|frame| ui::render(app, frame))?;
        tokio::select! {
            () = shutdown.cancelled() => {
                tracing::info!("arrêt demandé par un signal système");
                return Ok(());
            }
            event = terminal_events.next() => match event {
                Some(Ok(event)) => app.handle_terminal_event(&event),
                Some(Err(err)) => return Err(err.into()),
                None => return Ok(()),
            },
            Some(event) = event_rx.recv() => app.handle_app_event(event),
        }
    }
}
