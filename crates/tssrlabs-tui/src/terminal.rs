//! Préparation et restauration du terminal.

use ratatui::DefaultTerminal;

/// Session terminal en mode plein écran (mode brut + écran alternatif).
///
/// La restauration est garantie :
/// - à la destruction de la session (sortie normale ou erreur) ;
/// - en cas de panique, par le crochet installé par [`ratatui::try_init`],
///   qui restaure le terminal avant d'appeler le crochet précédent.
#[derive(Debug)]
pub struct TerminalSession {
    terminal: DefaultTerminal,
}

impl TerminalSession {
    /// Passe le terminal en mode plein écran.
    pub fn enter() -> std::io::Result<Self> {
        match ratatui::try_init() {
            Ok(terminal) => Ok(Self { terminal }),
            Err(err) => {
                // L'initialisation a pu échouer après le passage en mode brut :
                // on restaure quand même, la session n'existant pas pour le faire.
                let _ = ratatui::try_restore();
                Err(err)
            }
        }
    }

    /// Accès au terminal Ratatui.
    pub fn terminal_mut(&mut self) -> &mut DefaultTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        if let Err(err) = ratatui::try_restore() {
            tracing::error!(%err, "restauration du terminal impossible");
        }
    }
}
