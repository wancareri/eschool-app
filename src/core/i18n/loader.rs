use std::collections::HashMap;
use super::translations::Translations;

pub struct I18n {
    translations: HashMap<String, Translations>,
    current_lang: String,
}

impl I18n {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        
        // Load embedded translations
        translations.insert("en".to_string(), include_str!("../../../resource/locales/en/app.toml"));
        translations.insert("ru".to_string(), include_str!("../../../resource/locales/ru/app.toml"));
        translations.insert("be".to_string(), include_str!("../../../resource/locales/be/app.toml"));
        
        Self {
            translations: translations.into_iter()
                .map(|(k, v)| (k, toml::from_str(v).unwrap()))
                .collect(),
            current_lang: "en".to_string(),
        }
    }

    pub fn set_language(&mut self, lang: &str) {
        if self.translations.contains_key(lang) {
            self.current_lang = lang.to_string();
        }
    }

    pub fn t(&self) -> &Translations {
        self.translations.get(&self.current_lang)
            .unwrap_or(self.translations.get("en").unwrap())
    }

    pub fn available_languages(&self) -> Vec<&str> {
        self.translations.keys().map(|s| s.as_str()).collect()
    }
}
