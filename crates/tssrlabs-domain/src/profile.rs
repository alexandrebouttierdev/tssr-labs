//! Profils locaux d'apprenants.
//!
//! Aucun compte distant : un profil est une entrée locale qui regroupe
//! progression, notes et préférences.

use jiff::Timestamp;

/// Nombre maximal de caractères d'un nom de profil.
pub const MAX_NAME_CHARS: usize = 32;

/// Identifiant d'un profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProfileId(i64);

impl ProfileId {
    /// Construit un identifiant (attribué par le stockage).
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Valeur brute.
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// Raison du refus d'un nom de profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ProfileNameError {
    /// Nom vide (ou uniquement des espaces).
    #[error("le nom est vide")]
    Empty,
    /// Nom trop long.
    #[error("le nom dépasse {max} caractères")]
    TooLong {
        /// Longueur maximale autorisée.
        max: usize,
    },
    /// Caractère de contrôle (séquence d'échappement, etc.).
    #[error("le nom contient un caractère non imprimable")]
    ControlCharacter,
}

/// Nom de profil validé et normalisé.
///
/// Normalisation : espaces de début/fin retirés, espaces internes
/// consécutifs réduits à un seul. L'unicité est vérifiée sans tenir compte
/// de la casse (voir [`ProfileName::key`]).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileName(String);

impl ProfileName {
    /// Valide et normalise un nom saisi.
    pub fn parse(raw: &str) -> Result<Self, ProfileNameError> {
        let normalized = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if normalized.is_empty() {
            return Err(ProfileNameError::Empty);
        }
        if normalized.chars().any(char::is_control) {
            return Err(ProfileNameError::ControlCharacter);
        }
        if normalized.chars().count() > MAX_NAME_CHARS {
            return Err(ProfileNameError::TooLong {
                max: MAX_NAME_CHARS,
            });
        }
        Ok(Self(normalized))
    }

    /// Nom tel qu'affiché.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Clé d'unicité, insensible à la casse.
    pub fn key(&self) -> String {
        self.0.to_lowercase()
    }

    /// Vérifie une saisie de confirmation (suppression) : elle doit
    /// reproduire exactement le nom, aux espaces superflus près.
    pub fn matches_confirmation(&self, typed: &str) -> bool {
        Self::parse(typed).is_ok_and(|typed| typed == *self)
    }
}

impl std::fmt::Display for ProfileName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Profil local.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Profile {
    /// Identifiant.
    pub id: ProfileId,
    /// Nom.
    pub name: ProfileName,
    /// Date de création.
    pub created_at: Timestamp,
    /// Dernière activation, si le profil a déjà été utilisé.
    pub last_used_at: Option<Timestamp>,
}

/// Trie les profils : plus récemment utilisés d'abord, puis par nom.
pub fn sort_for_display(profiles: &mut [Profile]) {
    profiles.sort_by(|a, b| {
        b.last_used_at
            .cmp(&a.last_used_at)
            .then_with(|| a.name.key().cmp(&b.name.key()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_spaces_collapsed() {
        let name = ProfileName::parse("  Alice   Martin \t").unwrap();
        assert_eq!(name.as_str(), "Alice Martin");
    }

    #[test]
    fn invalid_names_are_rejected() {
        assert_eq!(ProfileName::parse("   "), Err(ProfileNameError::Empty));
        assert_eq!(
            ProfileName::parse("a\u{1b}[31m"),
            Err(ProfileNameError::ControlCharacter)
        );
        let long = "é".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(
            ProfileName::parse(&long),
            Err(ProfileNameError::TooLong {
                max: MAX_NAME_CHARS
            })
        );
        // Limite incluse, comptée en caractères et non en octets.
        assert!(ProfileName::parse(&"é".repeat(MAX_NAME_CHARS)).is_ok());
    }

    #[test]
    fn key_ignores_case() {
        let a = ProfileName::parse("Élodie").unwrap();
        let b = ProfileName::parse("éLODIE").unwrap();
        assert_eq!(a.key(), b.key());
    }

    #[test]
    fn confirmation_requires_exact_name() {
        let name = ProfileName::parse("Alice Martin").unwrap();
        assert!(name.matches_confirmation(" Alice  Martin "));
        assert!(!name.matches_confirmation("alice martin"));
        assert!(!name.matches_confirmation("Alice"));
        assert!(!name.matches_confirmation(""));
    }

    #[test]
    fn display_order_is_recent_first_then_name() {
        let profile = |id, name: &str, used: Option<i64>| Profile {
            id: ProfileId::new(id),
            name: ProfileName::parse(name).unwrap(),
            created_at: Timestamp::UNIX_EPOCH,
            last_used_at: used.map(|s| Timestamp::from_second(s).unwrap()),
        };
        let mut profiles = vec![
            profile(1, "zoé", None),
            profile(2, "Bob", Some(10)),
            profile(3, "alice", None),
            profile(4, "Carl", Some(20)),
        ];
        sort_for_display(&mut profiles);
        let ids: Vec<i64> = profiles.iter().map(|p| p.id.get()).collect();
        assert_eq!(ids, vec![4, 2, 3, 1]);
    }
}
