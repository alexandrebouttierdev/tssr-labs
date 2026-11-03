//! Couche de présentation Ratatui de TSSR Labs.
//!
//! Organisation (architecture de type Elm) :
//! - [`app::App`] : état de l'interface, modifié uniquement par
//!   [`app::App::update`] à partir d'une [`app::Action`] ;
//! - [`ui::render`] : dessin pur de l'état, sans effet de bord ;
//! - [`run::run`] : boucle d'événements asynchrone, qui ne bloque jamais sur
//!   une opération longue ;
//! - [`terminal::TerminalSession`] : préparation et restauration garantie du
//!   terminal, y compris en cas de panique.
//!
//! Cette crate ne contient aucune règle métier.

pub mod app;
pub mod effects;
pub mod i18n;
pub mod keymap;
pub mod layout;
pub mod run;
mod screens;
pub mod terminal;
mod text;
pub mod theme;
pub mod ui;
mod widgets;

/// Erreurs de la couche de présentation.
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    /// Erreur d'entrée/sortie du terminal.
    #[error("erreur du terminal")]
    Terminal(#[from] std::io::Error),
}
