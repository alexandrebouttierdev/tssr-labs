//! Couche application de TSSR Labs.
//!
//! - [`ports`] : interfaces (traits) que l'infrastructure implémente ;
//! - [`profiles`] : cas d'utilisation des profils et préférences.
//!
//! Cette couche orchestre le domaine ; elle ne connaît ni SQLite, ni le
//! terminal, ni Tokio (les futures sont exécutées par l'appelant).

pub mod ports;
pub mod profiles;

/// Services applicatifs disponibles pour la présentation.
#[derive(Debug, Clone)]
pub struct Services {
    /// Profils et préférences.
    pub profiles: profiles::ProfileService,
}
