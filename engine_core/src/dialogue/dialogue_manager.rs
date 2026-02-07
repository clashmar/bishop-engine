// engine_core/src/dialogue/dialogue_manager.rs
use crate::dialogue::*;
use rand::seq::SliceRandom;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manages dialogue loading, caching, and text selection.
pub struct DialogueManager {
    /// Base path for dialogue files.
    dialogue_root: PathBuf,
    /// Current language code.
    current_language: String,
    /// Available languages from manifest.
    available_languages: Vec<String>,
    /// Cached dialogue files by dialogue_id.
    cache: RefCell<HashMap<String, DialogueFile>>,
    /// State tracking for each (dialogue_id, key) pair.
    state: RefCell<HashMap<(String, String), DialogueState>>,
    /// Global dialogue configuration.
    pub config: DialogueConfig,
}

impl Default for DialogueManager {
    fn default() -> Self {
        Self {
            dialogue_root: PathBuf::new(),
            current_language: "en".to_string(),
            available_languages: vec!["en".to_string()],
            cache: RefCell::new(HashMap::new()),
            state: RefCell::new(HashMap::new()),
            config: DialogueConfig::default(),
        }
    }
}

impl DialogueManager {
    /// Creates a new DialogueManager with the given root path.
    pub fn new(dialogue_root: PathBuf) -> Self {
        let mut manager = Self {
            dialogue_root,
            ..Default::default()
        };
        manager.load_manifest();
        manager
    }

    /// Loads the manifest file to get available languages.
    fn load_manifest(&mut self) {
        let manifest_path = self.dialogue_root.join("_manifest.toml");
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = toml::from_str::<DialogueManifest>(&content) {
                self.current_language = manifest.default_language;
                self.available_languages = manifest.available;
            }
        }
    }

    /// Sets the current language. Returns false if language is not available.
    pub fn set_language(&mut self, lang: &str) -> bool {
        if self.available_languages.contains(&lang.to_string()) {
            self.current_language = lang.to_string();
            self.cache.borrow_mut().clear();
            true
        } else {
            false
        }
    }

    /// Returns the current language code.
    pub fn get_language(&self) -> &str {
        &self.current_language
    }

    /// Returns a list of available languages.
    pub fn get_languages(&self) -> &[String] {
        &self.available_languages
    }

    /// Loads a dialogue file by ID (filename without extension).
    fn load_dialogue_file(&self, dialogue_id: &str) -> bool {
        if self.cache.borrow().contains_key(dialogue_id) {
            return true;
        }

        let file_path = self
            .dialogue_root
            .join(&self.current_language)
            .join(format!("{}.toml", dialogue_id));

        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => return false,
        };

        let dialogue_file: DialogueFile = match toml::from_str(&content) {
            Ok(f) => f,
            Err(_) => return false,
        };

        self.cache.borrow_mut().insert(dialogue_id.to_string(), dialogue_file);
        true
    }

    /// Selects and returns text for the given dialogue_id and key.
    pub fn select_text(&self, dialogue_id: &str, key: &str) -> Option<String> {
        if !self.load_dialogue_file(dialogue_id) {
            return None;
        }

        let cache = self.cache.borrow();
        let file = cache.get(dialogue_id)?;
        let entry = file.entries.get(key)?.clone();
        drop(cache);

        if entry.variants.is_empty() {
            return entry.exhausted.clone();
        }

        let state_key = (dialogue_id.to_string(), key.to_string());
        let mut state_map = self.state.borrow_mut();
        let state = state_map.entry(state_key).or_default();

        Self::select_from_entry_static(&entry, state)
    }

    /// Selects text from an entry based on its selection mode.
    fn select_from_entry_static(entry: &DialogueEntry, state: &mut DialogueState) -> Option<String> {
        if entry.variants.is_empty() {
            return entry.exhausted.clone();
        }

        match entry.selection {
            SelectionMode::Random => {
                let mut rng = rand::thread_rng();
                let idx = rng.gen_range(0..entry.variants.len());
                Some(entry.variants[idx].clone())
            }
            SelectionMode::Sequential => {
                if entry.exhausted.is_some() {
                    if state.exhausted {
                        return entry.exhausted.clone();
                    }
                    let text = entry.variants.get(state.index).cloned();
                    state.index += 1;
                    if state.index >= entry.variants.len() {
                        state.exhausted = true;
                    }
                    text.or_else(|| entry.exhausted.clone())
                } else {
                    let text = entry.variants.get(state.index).cloned();
                    state.index = (state.index + 1) % entry.variants.len();
                    text
                }
            }
            SelectionMode::Once => {
                if state.exhausted {
                    return entry.exhausted.clone();
                }
                let text = entry.variants.get(state.index).cloned();
                state.index += 1;
                if state.index >= entry.variants.len() {
                    state.exhausted = true;
                }
                text.or_else(|| entry.exhausted.clone())
            }
            SelectionMode::Shuffle => {
                if entry.exhausted.is_some() && state.exhausted {
                    return entry.exhausted.clone();
                }

                if state.shuffle_order.is_empty()
                    || (entry.exhausted.is_none() && state.shuffle_index >= state.shuffle_order.len())
                {
                    let mut rng = rand::thread_rng();
                    state.shuffle_order = (0..entry.variants.len()).collect();
                    state.shuffle_order.shuffle(&mut rng);
                    state.shuffle_index = 0;
                }

                if state.shuffle_index >= state.shuffle_order.len() {
                    state.exhausted = true;
                    return entry.exhausted.clone();
                }

                let idx = state.shuffle_order[state.shuffle_index];
                state.shuffle_index += 1;

                if entry.exhausted.is_some() && state.shuffle_index >= state.shuffle_order.len() {
                    state.exhausted = true;
                }

                entry.variants.get(idx).cloned()
            }
        }
    }

    /// Resets state for a specific dialogue entry.
    pub fn reset_state(&self, dialogue_id: &str, key: &str) {
        let state_key = (dialogue_id.to_string(), key.to_string());
        if let Some(state) = self.state.borrow_mut().get_mut(&state_key) {
            state.reset();
        }
    }

    /// Clears all cached dialogue files.
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    /// Updates the dialogue root path.
    pub fn set_dialogue_root(&mut self, root: PathBuf) {
        self.dialogue_root = root;
        self.cache.borrow_mut().clear();
        self.state.borrow_mut().clear();
        self.load_manifest();
    }

    /// Returns the current dialogue root path.
    pub fn get_dialogue_root(&self) -> &Path {
        &self.dialogue_root
    }
}
