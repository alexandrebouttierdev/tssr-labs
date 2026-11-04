//! Écrans de l'application.

pub mod diagnostic;
pub mod exercise;
pub mod home;
pub mod library;
pub mod paths;
pub mod profiles;
pub mod progression;
pub mod reader;

use tssrlabs_domain::content::{Difficulty, Domain};

use crate::i18n::msg;

/// Identifiant du nom localisé d'un domaine.
pub fn domain_label_id(domain: Domain) -> &'static str {
    match domain {
        Domain::Linux => msg::DOMAIN_LINUX,
        Domain::Windows => msg::DOMAIN_WINDOWS,
        Domain::Network => msg::DOMAIN_NETWORK,
        Domain::Systems => msg::DOMAIN_SYSTEMS,
        Domain::Security => msg::DOMAIN_SECURITY,
        Domain::Virtualization => msg::DOMAIN_VIRTUALIZATION,
        Domain::Troubleshooting => msg::DOMAIN_TROUBLESHOOTING,
        Domain::Automation => msg::DOMAIN_AUTOMATION,
    }
}

/// Identifiant du nom localisé d'un niveau.
pub fn difficulty_label_id(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Beginner => msg::DIFFICULTY_BEGINNER,
        Difficulty::Intermediate => msg::DIFFICULTY_INTERMEDIATE,
        Difficulty::Advanced => msg::DIFFICULTY_ADVANCED,
    }
}
