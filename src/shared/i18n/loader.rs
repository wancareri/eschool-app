use std::collections::HashMap;
use super::translations::Translations;

pub struct I18n {
    translations: HashMap<String, Translations>,
    current_lang: String,
}

impl I18n {
    pub fn new() -> Self {
        let mut raw = HashMap::new();

        raw.insert("en".to_string(), include_str!("../../../resource/locales/en/app.toml"));
        raw.insert("ru".to_string(), include_str!("../../../resource/locales/ru/app.toml"));
        raw.insert("be".to_string(), include_str!("../../../resource/locales/be/app.toml"));

        let mut translations = HashMap::new();
        for (k, v) in raw {
            match toml::from_str(&v) {
                Ok(t) => {
                    translations.insert(k, t);
                }
                Err(e) => {
                    crate::shared::nslog::nslog(&format!("[i18n] Error parsing {k} locale: {e}"));
                }
            }
        }

        Self {
            translations,
            current_lang: "en".to_string(),
        }
    }

    pub fn set_language(&mut self, lang: &str) {
        if self.translations.contains_key(lang) {
            self.current_lang = lang.to_string();
        }
    }

    pub fn t(&self) -> &Translations {
        self.translations
            .get(&self.current_lang)
            .or_else(|| self.translations.get("en"))
            .or_else(|| self.translations.values().next())
            .expect("At least one translation catalog must be loaded")
    }

    pub fn available_languages(&self) -> Vec<&str> {
        self.translations.keys().map(|s| s.as_str()).collect()
    }
}
