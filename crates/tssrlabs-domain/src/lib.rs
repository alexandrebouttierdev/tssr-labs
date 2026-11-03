//! Domaine de TSSR Labs.
//!
//! Entités et règles métier pures : aucun accès disque, réseau, terminal,
//! base de données ou API externe. Les adaptateurs d'infrastructure
//! produisent ces types ; la présentation les affiche.

pub mod host;
