//! Adaptateurs d'infrastructure de TSSR Labs.
//!
//! Cette crate regroupe les accès aux systèmes externes : système de fichiers,
//! journaux locaux et, dans les tranches suivantes, SQLite, moteurs de
//! conteneurs, coffre de secrets, etc. Elle ne contient aucune règle métier.

pub mod host;
pub mod logging;
pub mod paths;
pub mod redact;
pub mod sqlite;
