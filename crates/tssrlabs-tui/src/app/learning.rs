//! État de la partie pédagogique : bibliothèque, parcours et lecture.

use std::cell::Cell;

use crossterm::event::{KeyCode, KeyEvent};
use tssrlabs_domain::content::{
    Catalog, Chapter, ContentId, Difficulty, LearningPath, LibraryQuery,
};

use super::{App, Screen};
use crate::keymap::Command;
use crate::widgets::input::TextInput;

/// Paliers du filtre de durée maximale (minutes).
pub const DURATION_STEPS: [Option<u16>; 5] = [None, Some(15), Some(30), Some(45), Some(60)];

/// État de la bibliothèque.
#[derive(Debug, Default)]
pub struct LibraryState {
    /// Critères courants.
    pub query: LibraryQuery,
    /// Champ de recherche.
    pub input: TextInput,
    /// La saisie de recherche est active.
    pub searching: bool,
    /// Ligne sélectionnée dans les résultats.
    pub row: usize,
}

/// État de la lecture d'un chapitre.
#[derive(Debug)]
pub struct ReaderState {
    /// Chapitre affiché.
    pub chapter: ContentId,
    /// Écran auquel revenir.
    pub return_to: Screen,
    /// Première ligne affichée.
    pub scroll: u16,
    /// Défilement maximal, mis à jour au rendu (dépend de la largeur).
    pub max_scroll: Cell<u16>,
    /// Hauteur visible, mise à jour au rendu.
    pub page: Cell<u16>,
}

fn next_in<T: Copy + PartialEq>(options: &[T], current: T) -> T {
    let index = options.iter().position(|o| *o == current).unwrap_or(0);
    options[(index + 1) % options.len()]
}

impl App {
    /// Catalogue de contenu.
    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    /// État de la bibliothèque.
    pub fn library(&self) -> &LibraryState {
        &self.library
    }

    /// Chapitres correspondant aux critères de la bibliothèque.
    pub fn library_results(&self) -> Vec<&Chapter> {
        self.catalog.search(&self.library.query)
    }

    /// Chapitres des parcours, à plat et dans l'ordre (un chapitre peut
    /// apparaître dans plusieurs parcours).
    pub fn path_entries(&self) -> Vec<(&LearningPath, &Chapter)> {
        self.catalog
            .paths()
            .iter()
            .flat_map(|path| {
                path.chapters
                    .iter()
                    .filter_map(|id| self.catalog.chapter(id))
                    .map(move |chapter| (path, chapter))
            })
            .collect()
    }

    /// Ligne sélectionnée dans les parcours.
    pub fn paths_row(&self) -> usize {
        self.paths_row
    }

    /// Lecture en cours.
    pub fn reader(&self) -> Option<&ReaderState> {
        self.reader.as_ref()
    }

    /// Touche pendant la saisie de recherche : tout est du texte, sauf
    /// Entrée, Échap et flèche bas qui terminent la saisie.
    pub(super) fn on_search_key(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Down => self.library.searching = false,
            _ => {
                if self.library.input.handle(key) {
                    self.library.query.text = self.library.input.value();
                    self.library.row = 0;
                }
            }
        }
    }

    /// Commandes propres au contenu ; renvoie `true` si la commande a été traitée.
    pub(super) fn on_learning_command(&mut self, command: Command) -> bool {
        match (self.screen, command) {
            (Screen::Library, Command::Activate) => {
                if let Some(id) = self
                    .library_results()
                    .get(self.library.row)
                    .map(|c| c.id.clone())
                {
                    self.open_chapter(id, Screen::Library);
                }
            }
            (Screen::Paths, Command::Activate) => {
                if let Some(id) = self
                    .path_entries()
                    .get(self.paths_row)
                    .map(|(_, c)| c.id.clone())
                {
                    self.open_chapter(id, Screen::Paths);
                }
            }
            (Screen::Library, Command::FocusSearch) => self.library.searching = true,
            (Screen::Library, Command::CycleDomain) => {
                let mut options = vec![None];
                options.extend(self.catalog.domains().into_iter().map(Some));
                self.library.query.domain = next_in(&options, self.library.query.domain);
                self.library.row = 0;
            }
            (Screen::Library, Command::CycleDifficulty) => {
                let mut options = vec![None];
                options.extend(Difficulty::ALL.into_iter().map(Some));
                self.library.query.difficulty = next_in(&options, self.library.query.difficulty);
                self.library.row = 0;
            }
            (Screen::Library, Command::CycleDuration) => {
                self.library.query.max_minutes =
                    next_in(&DURATION_STEPS, self.library.query.max_minutes);
                self.library.row = 0;
            }
            (Screen::Library, Command::ClearFilters) => self.library = LibraryState::default(),
            (Screen::Library, Command::Up | Command::Down) => {
                let count = self.library_results().len();
                self.library.row = step(self.library.row, count, command == Command::Down);
            }
            (Screen::Paths, Command::Up | Command::Down) => {
                let count = self.path_entries().len();
                self.paths_row = step(self.paths_row, count, command == Command::Down);
            }
            (Screen::Reader, _) => return self.on_reader_command(command),
            _ => return false,
        }
        true
    }

    fn on_reader_command(&mut self, command: Command) -> bool {
        let Some(reader) = &mut self.reader else {
            return false;
        };
        let max = reader.max_scroll.get();
        let page = reader.page.get().saturating_sub(1).max(1);
        reader.scroll = match command {
            Command::Up => reader.scroll.saturating_sub(1),
            Command::Down => reader.scroll.saturating_add(1).min(max),
            Command::PageUp => reader.scroll.saturating_sub(page),
            Command::PageDown => reader.scroll.saturating_add(page).min(max),
            Command::Top => 0,
            Command::Bottom => max,
            Command::Close => {
                self.close_reader(0);
                return true;
            }
            Command::NextScreen | Command::PrevScreen => {
                self.close_reader(if command == Command::NextScreen {
                    1
                } else {
                    -1
                });
                return true;
            }
            _ => return false,
        };
        true
    }

    fn open_chapter(&mut self, chapter: ContentId, return_to: Screen) {
        self.reader = Some(ReaderState {
            chapter,
            return_to,
            scroll: 0,
            max_scroll: Cell::new(0),
            page: Cell::new(1),
        });
        self.screen = Screen::Reader;
    }

    /// Quitte la lecture et revient à l'écran d'origine (décalé de `delta`).
    fn close_reader(&mut self, delta: isize) {
        let return_to = self.reader.take().map_or(Screen::Library, |r| r.return_to);
        self.screen = return_to.offset(delta);
    }
}

fn step(row: usize, count: usize, down: bool) -> usize {
    if down {
        (row + 1).min(count.saturating_sub(1))
    } else {
        row.saturating_sub(1)
    }
}
