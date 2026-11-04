//! Masquage des secrets dans le texte destiné aux journaux.
//!
//! Il s'agit d'une **défense en profondeur** : la règle principale reste de ne
//! jamais journaliser de secret (les secrets seront manipulés via des types
//! dédiés dont l'affichage est masqué). Ce filtre rattrape les fuites
//! accidentelles les plus courantes : clés d'API à préfixe connu, jetons
//! `Bearer`, paires `password=…`, identifiants dans une URL et clés privées PEM.
//!
//! Limite connue : un secret sans forme reconnaissable (mot de passe isolé dans
//! une phrase, par exemple) ne peut pas être détecté.

use std::borrow::Cow;
use std::sync::LazyLock;

use regex::Regex;

/// Texte de remplacement d'une valeur masquée.
pub const MASK: &str = "[MASQUÉ]";

/// Texte renvoyé si les filtres ne peuvent pas être construits : on préfère
/// perdre la ligne de journal que risquer d'écrire un secret.
const FILTER_UNAVAILABLE: &str = "[ligne masquée : filtre de secrets indisponible]";

/// Règles appliquées dans l'ordre. L'ordre compte : `Bearer` doit être traité
/// avant la règle générique `authorization: …`.
const RULES: &[(&str, &str)] = &[
    // Blocs de clés privées PEM/OpenSSH.
    (
        r"(?s)-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----.*?-----END [A-Z0-9 ]*PRIVATE KEY-----",
        "[CLÉ PRIVÉE MASQUÉE]",
    ),
    // Jetons Bearer (en-têtes HTTP).
    (r"(?i)\b(bearer)\s+[A-Za-z0-9._~+/=-]{8,}", "${1} [MASQUÉ]"),
    // Clés à préfixe connu : Anthropic, OpenAI et compatibles, GitHub, Slack, AWS.
    (
        r"\b(?:sk-ant-[A-Za-z0-9_-]{8,}|sk-[A-Za-z0-9_-]{16,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|xox[abpr]-[A-Za-z0-9-]{10,}|AKIA[0-9A-Z]{16})",
        "[MASQUÉ]",
    ),
    // Identifiants dans une URL : scheme://utilisateur:motdepasse@hôte
    (
        r"(?i)\b([a-z][a-z0-9+.-]*://)[^/\s:@]+:[^/\s@]+@",
        "${1}[MASQUÉ]@",
    ),
    // Paires clé/valeur sensibles : password=…, "api_key": "…", token: …
    (
        r#"(?i)\b(password|passwd|pwd|passphrase|secret|token|api[_-]?key|access[_-]?key|private[_-]?key|authorization)("?\s*[=:]\s*)("[^"]*"|'[^']*'|[^\s,;}]+)"#,
        "${1}${2}[MASQUÉ]",
    ),
];

static COMPILED: LazyLock<Result<Vec<(Regex, &'static str)>, regex::Error>> = LazyLock::new(|| {
    RULES
        .iter()
        .map(|(pattern, replacement)| Regex::new(pattern).map(|re| (re, *replacement)))
        .collect()
});

/// Renvoie `input` avec les secrets reconnus remplacés par [`MASK`].
///
/// Aucune allocation n'est faite si rien n'est masqué.
pub fn redact(input: &str) -> Cow<'_, str> {
    let rules = match COMPILED.as_ref() {
        Ok(rules) => rules,
        Err(_) => return Cow::Borrowed(FILTER_UNAVAILABLE),
    };
    let mut text = Cow::Borrowed(input);
    for (re, replacement) in rules {
        if let Cow::Owned(replaced) = re.replace_all(&text, *replacement) {
            text = Cow::Owned(replaced);
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_rules_compile() {
        assert!(COMPILED.is_ok(), "{:?}", COMPILED.as_ref().err());
    }

    #[test]
    fn text_without_secret_is_borrowed_unchanged() {
        let line = "démarrage du labo debian-01 (profil 3)";
        assert!(matches!(redact(line), Cow::Borrowed(s) if s == line));
    }

    #[test]
    fn masks_known_api_key_prefixes() {
        for secret in [
            "sk-ant-api03-AbCdEfGhIjKlMnOp",
            "sk-proj-AbCdEfGhIjKlMnOpQrSt",
            "ghp_0123456789abcdefghijABCDEFGHIJ",
            "github_pat_11ABCDEFG0123456789_abcdef",
            "AKIAABCDEFGHIJKLMNOP",
        ] {
            let line = format!("clé utilisée : {secret} ok");
            let out = redact(&line);
            assert!(!out.contains(secret), "non masqué : {out}");
            assert!(out.contains(MASK), "{out}");
            assert!(out.ends_with(" ok"), "{out}");
        }
    }

    #[test]
    fn masks_bearer_tokens_in_headers() {
        let out = redact("Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.abc.def");
        assert!(!out.contains("eyJhbGci"), "{out}");
    }

    #[test]
    fn masks_sensitive_key_value_pairs() {
        let cases = [
            ("password=hunter2 suite", "hunter2"),
            ("PASSWORD: 'mon secret'", "mon secret"),
            (r#"{"api_key": "abc123", "model": "x"}"#, "abc123"),
            ("token=xyz;next", "xyz"),
        ];
        for (input, secret) in cases {
            let out = redact(input);
            assert!(!out.contains(secret), "non masqué dans {out}");
            assert!(out.contains(MASK), "{out}");
        }
        // Le reste de la ligne est conservé.
        assert!(redact(r#"{"api_key": "abc123", "model": "x"}"#).contains(r#""model": "x""#));
    }

    #[test]
    fn masks_credentials_in_urls_but_keeps_host() {
        let out = redact("clonage de https://alice:s3cr3t@git.example.org/depot.git");
        assert!(!out.contains("s3cr3t"), "{out}");
        assert!(!out.contains("alice"), "{out}");
        assert!(
            out.contains("https://[MASQUÉ]@git.example.org/depot.git"),
            "{out}"
        );
    }

    #[test]
    fn masks_private_key_blocks() {
        let pem = "avant\n-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXk\n-----END OPENSSH PRIVATE KEY-----\naprès";
        let out = redact(pem);
        assert!(!out.contains("b3BlbnNzaC1rZXk"), "{out}");
        assert!(
            out.starts_with("avant\n") && out.ends_with("\naprès"),
            "{out}"
        );
    }
}
