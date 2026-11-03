//! État de l'interface.
//!
//! L'état évolue uniquement en réaction à des événements (clavier,
//! résultats d'effets). Toute opération d'entrée/sortie est demandée sous
//! forme d'[`Effect`] récupéré par la boucle via [`App::take_effects`].

use std::path::PathBuf;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use tssrlabs_application::profiles::{ProfileError, ProfilesSnapshot};
use tssrlabs_domain::host::HostInfo;
use tssrlabs_domain::preferences::Preferences;
use tssrlabs_domain::profile::{Profile, ProfileName, ProfileNameError};

use crate::effects::{AppEvent, Effect, ProfileOp};
use crate::i18n::{Language, Localizer, msg};
use crate::keymap::{Command, Keymap};
use crate::screens::diagnostic;
use crate::theme::{Theme, ThemeChoice};
use crate::widgets::input::TextInput;

/// Informations de contexte fournies par la racine de composition.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Version de l'application.
    pub version: &'static str,
    /// Hôte constaté au démarrage.
    pub host: HostInfo,
    /// Répertoire des données.
    pub data_dir: PathBuf,
    /// Répertoire de configuration.
    pub config_dir: PathBuf,
    /// Répertoire des journaux.
    pub log_dir: PathBuf,
    /// Fichier de la base de données.
    pub database_file: PathBuf,
}

/// Écrans disponibles. Seuls les écrans réellement implémentés figurent ici.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Tableau de bord.
    Home,
    /// Gestion des profils.
    Profiles,
    /// Diagnostic de l'hôte.
    Diagnostic,
}

impl Screen {
    /// Ordre de navigation.
    pub const ALL: [Self; 3] = [Self::Home, Self::Profiles, Self::Diagnostic];

    fn offset(self, delta: isize) -> Self {
        let len = Self::ALL.len() as isize;
        let index = Self::ALL.iter().position(|s| *s == self).unwrap_or(0) as isize;
        Self::ALL[(index + delta).rem_euclid(len) as usize]
    }

    /// Identifiant du titre localisé.
    pub fn title_id(self) -> &'static str {
        match self {
            Self::Home => msg::SCREEN_HOME,
            Self::Profiles => msg::SCREEN_PROFILES,
            Self::Diagnostic => msg::SCREEN_DIAGNOSTIC,
        }
    }
}

/// Objet d'une saisie de nom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamePurpose {
    /// Création d'un profil.
    Create,
    /// Renommage du profil indiqué.
    Rename(Profile),
}

/// Fenêtre superposée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    /// Aide (raccourcis).
    Help,
    /// Saisie d'un nom de profil.
    Name {
        /// Création ou renommage.
        purpose: NamePurpose,
        /// Champ de saisie.
        input: TextInput,
        /// Erreur à afficher.
        error: Option<String>,
    },
    /// Confirmation de suppression par saisie du nom.
    ConfirmDelete {
        /// Profil à supprimer.
        profile: Profile,
        /// Champ de saisie.
        input: TextInput,
        /// Erreur à afficher.
        error: Option<String>,
    },
}

impl Modal {
    fn is_dialog(&self) -> bool {
        !matches!(self, Self::Help)
    }

    fn set_error(&mut self, message: String) {
        if let Self::Name { error, .. } | Self::ConfirmDelete { error, .. } = self {
            *error = Some(message);
        }
    }
}

/// Message affiché sous l'écran jusqu'à la prochaine touche.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Notice {
    /// Information (succès).
    Info(String),
    /// Erreur.
    Error(String),
}

/// État de l'interface.
#[derive(Debug)]
pub struct App {
    running: bool,
    screen: Screen,
    modal: Option<Modal>,
    notice: Option<Notice>,
    theme_kind: ThemeChoice,
    diagnostic_row: usize,
    profile_row: usize,
    /// `None` tant que le chargement initial n'a pas abouti.
    profiles: Option<ProfilesSnapshot>,
    busy: bool,
    effects: Vec<Effect>,
    env: Environment,
    keymap: Keymap,
    i18n: Localizer,
}

impl App {
    /// Crée l'état initial et demande le chargement des profils.
    pub fn new(env: Environment) -> Self {
        Self {
            running: true,
            screen: Screen::Home,
            modal: None,
            notice: None,
            theme_kind: ThemeChoice::default(),
            diagnostic_row: 0,
            profile_row: 0,
            profiles: None,
            busy: true,
            effects: vec![Effect::LoadProfiles],
            env,
            keymap: Keymap::default(),
            i18n: Localizer::new(Language::default()),
        }
    }

    // --- Accès en lecture (rendu) -------------------------------------

    /// Indique si la boucle d'événements doit continuer.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Écran affiché.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Fenêtre superposée ouverte.
    pub fn modal(&self) -> Option<&Modal> {
        self.modal.as_ref()
    }

    /// Indique si l'aide est affichée.
    pub fn help_open(&self) -> bool {
        self.modal == Some(Modal::Help)
    }

    /// Message courant.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// Une opération est en cours.
    pub fn is_busy(&self) -> bool {
        self.busy
    }

    /// État des profils (`None` pendant le chargement).
    pub fn profiles(&self) -> Option<&ProfilesSnapshot> {
        self.profiles.as_ref()
    }

    /// Profil actif.
    pub fn active_profile(&self) -> Option<&Profile> {
        self.profiles
            .as_ref()
            .and_then(|s| s.active.as_ref())
            .map(|a| &a.profile)
    }

    /// Ligne sélectionnée dans la liste des profils.
    pub fn profile_row(&self) -> usize {
        self.profile_row
    }

    /// Ligne sélectionnée dans le diagnostic.
    pub fn diagnostic_row(&self) -> usize {
        self.diagnostic_row
    }

    /// Contexte d'exécution.
    pub fn env(&self) -> &Environment {
        &self.env
    }

    /// Raccourcis clavier.
    pub fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    /// Traducteur.
    pub fn i18n(&self) -> &Localizer {
        &self.i18n
    }

    /// Thème effectif, adapté aux capacités du terminal.
    pub fn theme(&self) -> Theme {
        Theme::resolve(self.theme_kind, self.env.host.terminal.color)
    }

    /// Indique si le changement de thème a un effet (terminal en couleurs).
    pub fn can_change_theme(&self) -> bool {
        self.env.host.terminal.color.has_colors()
    }

    /// Effets demandés depuis le dernier appel.
    pub fn take_effects(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.effects)
    }

    // --- Événements terminal ------------------------------------------

    /// Traite un événement du terminal.
    pub fn handle_terminal_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            self.on_key(key);
        }
        // Redimensionnement, focus : le prochain rendu suffit.
    }

    fn on_key(&mut self, key: &KeyEvent) {
        // Sous Windows, crossterm signale aussi les relâchements de touche.
        if key.kind != KeyEventKind::Press {
            return;
        }
        // Ctrl+C quitte toujours, même pendant une saisie.
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'C'))
        {
            self.running = false;
            return;
        }
        self.notice = None;
        if self.modal.as_ref().is_some_and(Modal::is_dialog) {
            self.on_dialog_key(key);
        } else if let Some(command) = self.keymap.command_for(key) {
            self.on_command(command);
        }
    }

    fn on_dialog_key(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Enter if !self.busy => self.submit_dialog(),
            KeyCode::Esc if !self.busy => self.modal = None,
            _ if !self.busy => {
                if let Some(
                    Modal::Name { input, error, .. } | Modal::ConfirmDelete { input, error, .. },
                ) = &mut self.modal
                    && input.handle(key)
                {
                    *error = None;
                }
            }
            _ => {}
        }
    }

    fn on_command(&mut self, command: Command) {
        if self.help_open() {
            match command {
                Command::Quit => self.running = false,
                Command::Close | Command::ToggleHelp => self.modal = None,
                _ => {}
            }
            return;
        }
        match command {
            Command::Quit => self.running = false,
            Command::ToggleHelp => self.modal = Some(Modal::Help),
            Command::Close => {}
            Command::NextScreen => self.screen = self.screen.offset(1),
            Command::PrevScreen => self.screen = self.screen.offset(-1),
            Command::CycleTheme => self.cycle_theme(),
            Command::Up | Command::Down => self.move_selection(command == Command::Down),
            Command::Activate => self.activate_selected(),
            Command::NewProfile => self.open_dialog_create(),
            Command::RenameProfile => self.open_dialog_rename(),
            Command::DeleteProfile => self.open_dialog_delete(),
        }
    }

    fn cycle_theme(&mut self) {
        if !self.can_change_theme() {
            return;
        }
        self.theme_kind = self.theme_kind.next();
        if let Some(profile) = self.active_profile() {
            let id = profile.id;
            self.effects.push(Effect::SavePreferences {
                id,
                preferences: Preferences {
                    theme: self.theme_kind,
                },
            });
        }
    }

    fn move_selection(&mut self, down: bool) {
        let (row, count) = match self.screen {
            Screen::Diagnostic => (&mut self.diagnostic_row, diagnostic::ROW_COUNT),
            Screen::Profiles => (
                &mut self.profile_row,
                self.profiles.as_ref().map_or(0, |s| s.profiles.len()),
            ),
            Screen::Home => return,
        };
        *row = if down {
            (*row + 1).min(count.saturating_sub(1))
        } else {
            row.saturating_sub(1)
        };
    }

    /// Profil sélectionné, si l'écran Profils est utilisable.
    fn selected_profile(&self) -> Option<Profile> {
        if self.screen != Screen::Profiles || self.busy {
            return None;
        }
        self.profiles
            .as_ref()
            .and_then(|s| s.profiles.get(self.profile_row))
            .cloned()
    }

    fn activate_selected(&mut self) {
        let Some(profile) = self.selected_profile() else {
            return;
        };
        if self.active_profile().map(|p| p.id) == Some(profile.id) {
            self.screen = Screen::Home;
        } else {
            self.busy = true;
            self.effects.push(Effect::SelectProfile(profile.id));
        }
    }

    fn open_dialog_create(&mut self) {
        if self.screen == Screen::Profiles && !self.busy && self.profiles.is_some() {
            self.modal = Some(Modal::Name {
                purpose: NamePurpose::Create,
                input: TextInput::default(),
                error: None,
            });
        }
    }

    fn open_dialog_rename(&mut self) {
        if let Some(profile) = self.selected_profile() {
            self.modal = Some(Modal::Name {
                input: TextInput::with_value(profile.name.as_str()),
                purpose: NamePurpose::Rename(profile),
                error: None,
            });
        }
    }

    fn open_dialog_delete(&mut self) {
        if let Some(profile) = self.selected_profile() {
            self.modal = Some(Modal::ConfirmDelete {
                profile,
                input: TextInput::default(),
                error: None,
            });
        }
    }

    fn submit_dialog(&mut self) {
        let effect = match &self.modal {
            Some(Modal::Name { purpose, input, .. }) => {
                let name = input.value();
                // Vérification immédiate des règles du domaine ; le service
                // revalide de toute façon.
                if let Err(error) = ProfileName::parse(&name) {
                    let text = name_error_text(&self.i18n, error);
                    if let Some(modal) = &mut self.modal {
                        modal.set_error(text);
                    }
                    return;
                }
                match purpose {
                    NamePurpose::Create => Effect::CreateProfile { name },
                    NamePurpose::Rename(profile) => Effect::RenameProfile {
                        id: profile.id,
                        name,
                    },
                }
            }
            Some(Modal::ConfirmDelete { profile, input, .. }) => {
                if !profile.name.matches_confirmation(&input.value()) {
                    let text = self.i18n.t(msg::ERROR_CONFIRMATION);
                    if let Some(modal) = &mut self.modal {
                        modal.set_error(text);
                    }
                    return;
                }
                Effect::DeleteProfile {
                    id: profile.id,
                    confirmation: input.value(),
                }
            }
            _ => return,
        };
        self.busy = true;
        self.effects.push(effect);
    }

    // --- Résultats des effets -----------------------------------------

    /// Traite le résultat d'un effet.
    pub fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Profiles { op, result } => {
                self.busy = false;
                match result {
                    Ok(snapshot) => self.on_profiles_updated(op, snapshot),
                    Err(error) => self.on_profile_error(op, &error),
                }
            }
            AppEvent::Deleted(result) => {
                self.busy = false;
                match result {
                    Ok(deletion) => {
                        self.modal = None;
                        self.apply_snapshot(deletion.snapshot);
                        self.notice = Some(Notice::Info(self.i18n.t_args(
                            msg::STATUS_DELETED,
                            &[("path", deletion.backup.display().to_string().into())],
                        )));
                    }
                    Err(error) => self.show_error(&error),
                }
            }
            AppEvent::PreferencesSaved(Err(error)) => {
                let detail = profile_error_text(&self.i18n, &error);
                self.notice = Some(Notice::Error(
                    self.i18n
                        .t_args(msg::STATUS_PREFS_ERROR, &[("detail", detail.into())]),
                ));
            }
            AppEvent::PreferencesSaved(Ok(())) => {}
        }
    }

    fn on_profiles_updated(&mut self, op: ProfileOp, snapshot: ProfilesSnapshot) {
        let renamed = match op {
            ProfileOp::Rename(id) => snapshot
                .profiles
                .iter()
                .find(|p| p.id == id)
                .map(|p| p.name.to_string()),
            _ => None,
        };
        self.apply_snapshot(snapshot);
        let active_name = self.active_profile().map(|p| p.name.to_string());
        match op {
            ProfileOp::Load => {
                if active_name.is_some() {
                    self.screen = Screen::Home;
                } else {
                    self.screen = Screen::Profiles;
                    if self
                        .profiles
                        .as_ref()
                        .is_some_and(|s| s.profiles.is_empty())
                    {
                        self.open_dialog_create();
                    }
                }
            }
            ProfileOp::Create | ProfileOp::Select => {
                self.modal = None;
                self.screen = Screen::Home;
                let id = if op == ProfileOp::Create {
                    msg::STATUS_CREATED
                } else {
                    msg::STATUS_SELECTED
                };
                if let Some(name) = active_name {
                    self.notice =
                        Some(Notice::Info(self.i18n.t_args(id, &[("name", name.into())])));
                }
            }
            ProfileOp::Rename(_) => {
                self.modal = None;
                if let Some(name) = renamed {
                    self.notice = Some(Notice::Info(
                        self.i18n
                            .t_args(msg::STATUS_RENAMED, &[("name", name.into())]),
                    ));
                }
            }
        }
    }

    fn on_profile_error(&mut self, op: ProfileOp, error: &ProfileError) {
        if op == ProfileOp::Load {
            // Rien n'a pu être lu : liste vide pour rester utilisable, et
            // message explicite.
            self.profiles = Some(ProfilesSnapshot::default());
            self.screen = Screen::Profiles;
        }
        self.show_error(error);
    }

    /// Affiche l'erreur dans la fenêtre ouverte, sinon sous l'écran.
    fn show_error(&mut self, error: &ProfileError) {
        let text = profile_error_text(&self.i18n, error);
        match &mut self.modal {
            Some(modal) if modal.is_dialog() => modal.set_error(text),
            _ => self.notice = Some(Notice::Error(text)),
        }
    }

    fn apply_snapshot(&mut self, snapshot: ProfilesSnapshot) {
        if let Some(active) = &snapshot.active {
            self.theme_kind = active.preferences.theme;
        }
        // Garde la sélection sur le profil actif après une opération.
        self.profile_row = snapshot
            .active
            .as_ref()
            .and_then(|a| snapshot.profiles.iter().position(|p| p.id == a.profile.id))
            .unwrap_or(0)
            .min(snapshot.profiles.len().saturating_sub(1));
        if snapshot.active.is_none() && self.screen == Screen::Home {
            self.screen = Screen::Profiles;
        }
        self.profiles = Some(snapshot);
    }
}

/// Texte localisé d'une erreur de nom.
pub fn name_error_text(i18n: &Localizer, error: ProfileNameError) -> String {
    match error {
        ProfileNameError::Empty => i18n.t(msg::ERROR_NAME_EMPTY),
        ProfileNameError::TooLong { max } => {
            i18n.t_args(msg::ERROR_NAME_TOO_LONG, &[("max", max.into())])
        }
        ProfileNameError::ControlCharacter => i18n.t(msg::ERROR_NAME_CONTROL),
    }
}

/// Texte localisé d'une erreur de profil.
pub fn profile_error_text(i18n: &Localizer, error: &ProfileError) -> String {
    match error {
        ProfileError::InvalidName(e) => name_error_text(i18n, *e),
        ProfileError::NameTaken => i18n.t(msg::ERROR_NAME_TAKEN),
        ProfileError::NotFound => i18n.t(msg::ERROR_NOT_FOUND),
        ProfileError::ConfirmationMismatch => i18n.t(msg::ERROR_CONFIRMATION),
        ProfileError::Storage(e) => {
            i18n.t_args(msg::ERROR_STORAGE, &[("detail", e.to_string().into())])
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crossterm::event::KeyEventState;
    use jiff::Timestamp;
    use tssrlabs_application::ports::RepositoryError;
    use tssrlabs_application::profiles::{ActiveProfile, Deletion};
    use tssrlabs_domain::host::{ColorSupport, DetectionSource, TerminalInfo};
    use tssrlabs_domain::profile::ProfileId;

    pub(crate) fn test_env(color: ColorSupport) -> Environment {
        Environment {
            version: "9.9.9",
            host: HostInfo {
                os: "linux".into(),
                arch: "x86_64".into(),
                terminal: TerminalInfo {
                    term: Some("xterm-256color".into()),
                    program: Some("WezTerm".into()),
                    color,
                    color_source: DetectionSource::Detected,
                },
            },
            data_dir: "/home/apprenant/.local/share/tssrlabs".into(),
            config_dir: "/home/apprenant/.config/tssrlabs".into(),
            log_dir: "/home/apprenant/.local/share/tssrlabs/logs".into(),
            database_file: "/home/apprenant/.local/share/tssrlabs/tssrlabs.sqlite3".into(),
        }
    }

    pub(crate) fn profile(id: i64, name: &str) -> Profile {
        Profile {
            id: ProfileId::new(id),
            name: ProfileName::parse(name).unwrap(),
            created_at: Timestamp::from_second(1_700_000_000).unwrap(),
            last_used_at: None,
        }
    }

    pub(crate) fn snapshot(
        names: &[&str],
        active: Option<usize>,
        theme: ThemeChoice,
    ) -> ProfilesSnapshot {
        let profiles: Vec<Profile> = names
            .iter()
            .enumerate()
            .map(|(i, n)| profile(i as i64 + 1, n))
            .collect();
        ProfilesSnapshot {
            active: active.map(|i| ActiveProfile {
                profile: profiles[i].clone(),
                preferences: Preferences { theme },
            }),
            profiles,
        }
    }

    /// Application chargée avec les profils indiqués.
    pub(crate) fn loaded_app(names: &[&str], active: Option<usize>) -> App {
        let mut app = App::new(test_env(ColorSupport::TrueColor));
        assert_eq!(app.take_effects(), vec![Effect::LoadProfiles]);
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Load,
            result: Ok(snapshot(names, active, ThemeChoice::Dark)),
        });
        app
    }

    fn press(code: KeyCode) -> Event {
        press_with(code, KeyModifiers::NONE)
    }

    fn press_with(code: KeyCode, modifiers: KeyModifiers) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn send(app: &mut App, code: KeyCode) {
        app.handle_terminal_event(&press(code));
    }

    fn type_text(app: &mut App, text: &str) {
        for c in text.chars() {
            send(app, KeyCode::Char(c));
        }
    }

    #[test]
    fn starts_by_loading_profiles() {
        let mut app = App::new(test_env(ColorSupport::TrueColor));
        assert!(app.is_busy());
        assert_eq!(app.profiles(), None);
        assert_eq!(app.take_effects(), vec![Effect::LoadProfiles]);
        assert!(app.take_effects().is_empty());
    }

    #[test]
    fn first_launch_opens_profile_creation() {
        let app = loaded_app(&[], None);
        assert_eq!(app.screen(), Screen::Profiles);
        assert!(matches!(
            app.modal(),
            Some(Modal::Name {
                purpose: NamePurpose::Create,
                ..
            })
        ));
    }

    #[test]
    fn remembered_profile_goes_to_dashboard_with_its_theme() {
        let mut app = App::new(test_env(ColorSupport::TrueColor));
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Load,
            result: Ok(snapshot(&["Alice"], Some(0), ThemeChoice::Colorblind)),
        });
        assert_eq!(app.screen(), Screen::Home);
        assert_eq!(app.theme().kind, ThemeChoice::Colorblind);
        assert_eq!(app.active_profile().unwrap().name.as_str(), "Alice");
    }

    #[test]
    fn typing_a_name_does_not_trigger_shortcuts() {
        let mut app = loaded_app(&[], None);
        type_text(&mut app, "qtns");
        assert!(app.is_running());
        send(&mut app, KeyCode::Enter);
        assert_eq!(
            app.take_effects(),
            vec![Effect::CreateProfile {
                name: "qtns".into()
            }]
        );
        assert!(app.is_busy());
    }

    #[test]
    fn invalid_name_is_reported_without_effect() {
        let mut app = loaded_app(&[], None);
        type_text(&mut app, "   ");
        send(&mut app, KeyCode::Enter);
        assert!(app.take_effects().is_empty());
        assert!(
            matches!(app.modal(), Some(Modal::Name { error: Some(e), .. }) if e.contains("vide"))
        );
    }

    #[test]
    fn creation_success_closes_dialog_and_shows_dashboard() {
        let mut app = loaded_app(&[], None);
        type_text(&mut app, "Alice");
        send(&mut app, KeyCode::Enter);
        app.take_effects();
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Create,
            result: Ok(snapshot(&["Alice"], Some(0), ThemeChoice::Dark)),
        });
        assert_eq!(app.modal(), None);
        assert_eq!(app.screen(), Screen::Home);
        assert!(matches!(app.notice(), Some(Notice::Info(t)) if t.contains("Alice")));
    }

    #[test]
    fn duplicate_name_error_stays_in_dialog() {
        let mut app = loaded_app(&[], None);
        type_text(&mut app, "Alice");
        send(&mut app, KeyCode::Enter);
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Create,
            result: Err(ProfileError::NameTaken),
        });
        assert!(!app.is_busy());
        assert!(
            matches!(app.modal(), Some(Modal::Name { error: Some(e), .. }) if e.contains("déjà"))
        );
    }

    #[test]
    fn escape_cancels_dialog_but_not_while_busy() {
        let mut app = loaded_app(&[], None);
        type_text(&mut app, "A");
        send(&mut app, KeyCode::Enter);
        send(&mut app, KeyCode::Esc);
        assert!(
            app.modal().is_some(),
            "pas d'annulation pendant l'opération"
        );
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Create,
            result: Err(ProfileError::Storage(RepositoryError::Unavailable(
                "x".into(),
            ))),
        });
        send(&mut app, KeyCode::Esc);
        assert_eq!(app.modal(), None);
    }

    #[test]
    fn ctrl_c_quits_even_in_dialog() {
        let mut app = loaded_app(&[], None);
        app.handle_terminal_event(&press_with(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(!app.is_running());
    }

    #[test]
    fn select_rename_and_delete_from_profiles_screen() {
        let mut app = loaded_app(&["Alice", "Bob"], Some(0));
        send(&mut app, KeyCode::Tab);
        assert_eq!(app.screen(), Screen::Profiles);

        send(&mut app, KeyCode::Down);
        send(&mut app, KeyCode::Enter);
        assert_eq!(
            app.take_effects(),
            vec![Effect::SelectProfile(ProfileId::new(2))]
        );
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Select,
            result: Ok(snapshot(&["Alice", "Bob"], Some(1), ThemeChoice::Dark)),
        });
        assert_eq!(app.screen(), Screen::Home);

        send(&mut app, KeyCode::Tab);
        assert_eq!(app.profile_row(), 1, "sélection sur le profil actif");
        send(&mut app, KeyCode::Char('r'));
        type_text(&mut app, "by");
        send(&mut app, KeyCode::Enter);
        assert_eq!(
            app.take_effects(),
            vec![Effect::RenameProfile {
                id: ProfileId::new(2),
                name: "Bobby".into()
            }]
        );
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Rename(ProfileId::new(2)),
            result: Ok(snapshot(&["Alice", "Bobby"], Some(1), ThemeChoice::Dark)),
        });

        send(&mut app, KeyCode::Char('s'));
        type_text(&mut app, "bobby");
        send(&mut app, KeyCode::Enter);
        assert!(
            app.take_effects().is_empty(),
            "confirmation sensible à la casse"
        );
        for _ in 0..5 {
            send(&mut app, KeyCode::Backspace);
        }
        type_text(&mut app, "Bobby");
        send(&mut app, KeyCode::Enter);
        assert_eq!(
            app.take_effects(),
            vec![Effect::DeleteProfile {
                id: ProfileId::new(2),
                confirmation: "Bobby".into()
            }]
        );
        app.handle_app_event(AppEvent::Deleted(Ok(Deletion {
            snapshot: snapshot(&["Alice"], None, ThemeChoice::Dark),
            backup: "/sauvegardes/copie.sqlite3".into(),
        })));
        assert_eq!(app.modal(), None);
        assert!(matches!(app.notice(), Some(Notice::Info(t)) if t.contains("copie.sqlite3")));
        assert_eq!(app.active_profile(), None);
    }

    #[test]
    fn profile_commands_are_ignored_outside_profiles_screen() {
        let mut app = loaded_app(&["Alice"], Some(0));
        for key in ['n', 'r', 's'] {
            send(&mut app, KeyCode::Char(key));
        }
        send(&mut app, KeyCode::Enter);
        assert_eq!(app.modal(), None);
        assert!(app.take_effects().is_empty());
    }

    #[test]
    fn theme_change_is_saved_for_active_profile_only() {
        let mut app = loaded_app(&["Alice"], Some(0));
        send(&mut app, KeyCode::Char('t'));
        assert_eq!(app.theme().kind, ThemeChoice::HighContrast);
        assert_eq!(
            app.take_effects(),
            vec![Effect::SavePreferences {
                id: ProfileId::new(1),
                preferences: Preferences {
                    theme: ThemeChoice::HighContrast
                }
            }]
        );

        let mut without = loaded_app(&["Alice"], None);
        send(&mut without, KeyCode::Esc); // ferme rien ; liste non vide
        send(&mut without, KeyCode::Char('t'));
        assert!(without.take_effects().is_empty());
    }

    #[test]
    fn theme_does_not_change_without_colors() {
        let mut app = App::new(test_env(ColorSupport::None));
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Load,
            result: Ok(snapshot(&["Alice"], Some(0), ThemeChoice::Dark)),
        });
        app.take_effects();
        send(&mut app, KeyCode::Char('t'));
        assert_eq!(app.theme().kind, ThemeChoice::Monochrome);
        assert!(app.take_effects().is_empty());
    }

    #[test]
    fn preference_save_failure_is_shown() {
        let mut app = loaded_app(&["Alice"], Some(0));
        app.handle_app_event(AppEvent::PreferencesSaved(Err(ProfileError::NotFound)));
        assert!(matches!(app.notice(), Some(Notice::Error(_))));
        send(&mut app, KeyCode::Down);
        assert_eq!(
            app.notice(),
            None,
            "le message disparaît à la touche suivante"
        );
    }

    #[test]
    fn load_failure_keeps_app_usable() {
        let mut app = App::new(test_env(ColorSupport::TrueColor));
        app.handle_app_event(AppEvent::Profiles {
            op: ProfileOp::Load,
            result: Err(ProfileError::Storage(RepositoryError::Backend(
                "disque".into(),
            ))),
        });
        assert_eq!(app.screen(), Screen::Profiles);
        assert!(matches!(app.notice(), Some(Notice::Error(t)) if t.contains("disque")));
        assert!(!app.is_busy());
    }

    #[test]
    fn help_blocks_other_commands() {
        let mut app = loaded_app(&["Alice"], Some(0));
        send(&mut app, KeyCode::F(1));
        assert!(app.help_open());
        send(&mut app, KeyCode::Tab);
        assert_eq!(app.screen(), Screen::Home);
        send(&mut app, KeyCode::Esc);
        assert!(!app.help_open());
    }

    #[test]
    fn diagnostic_selection_is_clamped() {
        let mut app = loaded_app(&["Alice"], Some(0));
        send(&mut app, KeyCode::BackTab);
        assert_eq!(app.screen(), Screen::Diagnostic);
        send(&mut app, KeyCode::Up);
        assert_eq!(app.diagnostic_row(), 0);
        for _ in 0..100 {
            send(&mut app, KeyCode::Char('j'));
        }
        assert_eq!(app.diagnostic_row(), diagnostic::ROW_COUNT - 1);
    }
}
