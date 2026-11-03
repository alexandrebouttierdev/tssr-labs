//! Analyse minimale de la ligne de commande.

use std::ffi::OsString;

/// Commande demandée.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Lancer l'interface (aucun argument).
    Run,
    /// Afficher l'aide.
    Help,
    /// Afficher la version.
    Version,
    /// Afficher les répertoires utilisés (données, configuration, journaux).
    Paths,
}

/// Erreur d'utilisation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CliError {
    /// Argument non reconnu.
    #[error("argument inconnu : {0}")]
    UnknownArgument(String),
    /// Plusieurs arguments fournis.
    #[error("un seul argument est accepté")]
    TooManyArguments,
}

/// Texte d'aide.
pub const HELP: &str = "\
TSSR Labs — plateforme pédagogique systèmes et réseaux en terminal

Utilisation :
  tssrlabs              lance l'interface
  tssrlabs --paths      affiche les répertoires de données, configuration et journaux
  tssrlabs --version    affiche la version
  tssrlabs --help       affiche cette aide

Variables d'environnement :
  TSSRLABS_HOME   regroupe toutes les données sous ce répertoire
  TSSRLABS_LOG    niveau des journaux locaux (ex. : debug, info, warn)

Aucune donnée n'est envoyée sur le réseau : pas de compte, pas de télémétrie.
";

/// Analyse les arguments (sans le nom du programme).
pub fn parse<I>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(Command::Run);
    };
    if args.next().is_some() {
        return Err(CliError::TooManyArguments);
    }
    match first.to_str() {
        Some("-h" | "--help") => Ok(Command::Help),
        Some("-V" | "--version") => Ok(Command::Version),
        Some("--paths") => Ok(Command::Paths),
        _ => Err(CliError::UnknownArgument(
            first.to_string_lossy().into_owned(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_strs(args: &[&str]) -> Result<Command, CliError> {
        parse(args.iter().map(OsString::from))
    }

    #[test]
    fn no_argument_runs_the_interface() {
        assert_eq!(parse_strs(&[]), Ok(Command::Run));
    }

    #[test]
    fn known_flags_are_recognized() {
        assert_eq!(parse_strs(&["--help"]), Ok(Command::Help));
        assert_eq!(parse_strs(&["-h"]), Ok(Command::Help));
        assert_eq!(parse_strs(&["--version"]), Ok(Command::Version));
        assert_eq!(parse_strs(&["-V"]), Ok(Command::Version));
        assert_eq!(parse_strs(&["--paths"]), Ok(Command::Paths));
    }

    #[test]
    fn unknown_or_extra_arguments_are_rejected() {
        assert_eq!(
            parse_strs(&["--debug"]),
            Err(CliError::UnknownArgument("--debug".into()))
        );
        assert_eq!(
            parse_strs(&["--help", "--version"]),
            Err(CliError::TooManyArguments)
        );
    }
}
