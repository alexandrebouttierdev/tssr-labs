//! Tests du binaire en conditions réelles (processus séparé).

// Fichier de test : un échec doit interrompre le test immédiatement.
#![allow(clippy::unwrap_used)]

use std::process::{Command, Output, Stdio};

fn tssrlabs(args: &[&str], home: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tssrlabs"))
        .args(args)
        .env("TSSRLABS_HOME", home)
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

#[test]
fn version_flag_prints_version() {
    let home = tempfile::tempdir().unwrap();
    let out = tssrlabs(&["--version"], home.path());
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(
        stdout.trim(),
        format!("tssrlabs {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_flag_prints_usage() {
    let home = tempfile::tempdir().unwrap();
    let out = tssrlabs(&["--help"], home.path());
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .contains("Utilisation")
    );
}

#[test]
fn paths_flag_uses_override_without_creating_anything() {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().join("racine");
    let out = tssrlabs(&["--paths"], &root);
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains(&root.join("logs").display().to_string()),
        "{stdout}"
    );
    assert!(!root.exists(), "--paths ne doit rien créer");
}

#[test]
fn unknown_argument_is_a_usage_error() {
    let home = tempfile::tempdir().unwrap();
    let out = tssrlabs(&["--inconnu"], home.path());
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("argument inconnu")
    );
}

#[test]
fn non_interactive_launch_fails_cleanly_without_side_effects() {
    let home = tempfile::tempdir().unwrap();
    let root = home.path().join("racine");
    let out = tssrlabs(&[], &root);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("terminal interactif"), "{stderr}");
    assert!(!root.exists(), "aucun répertoire ne doit être créé");
}
