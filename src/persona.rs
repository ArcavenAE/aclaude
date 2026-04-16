use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::error::{ForestageError, Result};

include!(concat!(env!("OUT_DIR"), "/themes_embedded.rs"));

/// OCEAN personality model scores (Big Five).
///
/// Retained for analytical correlation, not for directing the LLM.
/// The LLM embodies the character from training data; OCEAN scores
/// are used post-hoc to predict which personas perform well at which tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ocean {
    #[serde(alias = "O")]
    pub openness: f64,
    #[serde(alias = "C")]
    pub conscientiousness: f64,
    #[serde(alias = "E")]
    pub extraversion: f64,
    #[serde(alias = "A")]
    pub agreeableness: f64,
    #[serde(alias = "N")]
    pub neuroticism: f64,
}

/// Helper/sidekick character associated with a persona.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Helper {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub style: String,
}

/// A character in a theme's roster.
///
/// Characters are personas — dramatic masks the agent wears. The LLM
/// embodies the character from its training corpus; the card orients
/// and discriminates. All fields are preserved for their respective
/// uses (portraits, analysis, flavor, discrimination).
///
/// Characters are NOT team roles. A character can fill any role on any
/// team. See finding-019 (agent taxonomy) for the design rationale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// Full character name as it appears in the source material.
    pub character: String,
    /// Short display name / nickname.
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    /// Visual description for portrait generation.
    pub visual: Option<String>,
    /// OCEAN personality scores for analytical correlation.
    pub ocean: Option<Ocean>,
    /// Communication style — how they talk.
    #[serde(default)]
    pub style: String,
    /// Domain expertise from the source material (backstory flavor).
    #[serde(default)]
    pub expertise: String,
    /// Key personality trait — discrimination signal.
    #[serde(default)]
    pub r#trait: String,
    /// What role key they held in the pre-taxonomy system (migration reference).
    #[serde(default)]
    pub backstory_role: String,
    /// Prose description of their role in the source material.
    #[serde(default)]
    pub backstory_role_description: String,
    /// Behavioral quirks the LLM can embody.
    #[serde(default)]
    pub quirks: Vec<String>,
    /// Signature phrases from the source material.
    #[serde(default)]
    pub catchphrases: Vec<String>,
    /// Display emoji.
    pub emoji: Option<String>,
    /// Helper/sidekick character.
    pub helper: Option<Helper>,
}

/// Theme metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// Citation / provenance — where this theme comes from.
    /// e.g. "The Expanse by James S.A. Corey (2011-2021)"
    #[serde(default)]
    pub source: String,
    pub user_title: Option<String>,
    pub character_immersion: Option<String>,
    pub spinner_verbs: Option<Vec<String>>,
    pub dimensions: Option<HashMap<String, String>>,
}

/// Top-level theme file structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFile {
    #[serde(default)]
    pub category: String,
    pub theme: ThemeInfo,
    /// Character roster, keyed by character slug (e.g. "naomi-nagata").
    /// NOT keyed by team role. Any character can fill any role.
    #[serde(default)]
    pub characters: HashMap<String, Character>,
}

/// List all available theme slugs (from embedded data).
pub fn list_themes() -> Vec<String> {
    let themes = embedded_themes();
    let mut slugs: Vec<String> = themes.keys().map(ToString::to_string).collect();
    slugs.sort();
    slugs
}

/// Load a theme by slug.
pub fn load_theme(slug: &str) -> Result<ThemeFile> {
    let themes = embedded_themes();
    let yaml = themes
        .get(slug)
        .ok_or_else(|| ForestageError::ThemeNotFound {
            slug: slug.to_string(),
        })?;

    let theme: ThemeFile = serde_yaml::from_str(yaml).map_err(|e| ForestageError::Yaml {
        path: format!("embedded:{slug}"),
        source: e,
    })?;

    Ok(theme)
}

/// Get a character by slug from a theme's roster.
pub fn get_character<'a>(theme: &'a ThemeFile, slug: &str) -> Result<&'a Character> {
    theme
        .characters
        .get(slug)
        .ok_or_else(|| ForestageError::CharacterNotFound {
            character: slug.to_string(),
            theme: theme.theme.name.clone(),
        })
}

/// Backwards-compatible alias — get a character by the old role key.
/// Searches backstory_role fields for a match. Used during migration.
pub fn get_character_by_legacy_role<'a>(theme: &'a ThemeFile, role: &str) -> Result<&'a Character> {
    theme
        .characters
        .values()
        .find(|c| c.backstory_role == role)
        .ok_or_else(|| ForestageError::CharacterNotFound {
            character: format!("(legacy role: {role})"),
            theme: theme.theme.name.clone(),
        })
}

/// Build system prompt text based on immersion level.
///
/// The prompt orients the LLM on which character to embody. It does NOT
/// include team role or identity — those are separate concerns injected
/// by the caller (session.rs, bridge.rs).
pub fn build_system_prompt(theme: &ThemeFile, character: &Character, immersion: &str) -> String {
    match immersion {
        "high" => {
            let source = &theme.theme.source;
            let mut parts = vec![format!(
                "You are {}, from {}. {}.",
                character.character, source, character.style
            )];
            if !character.expertise.is_empty() {
                parts.push(format!("Background: {}", character.expertise));
            }
            if !character.r#trait.is_empty() {
                parts.push(format!("Key trait: {}", character.r#trait));
            }
            if !character.quirks.is_empty() {
                parts.push(format!("Quirks: {}", character.quirks.join("; ")));
            }
            if !character.catchphrases.is_empty() {
                parts.push(format!(
                    "Catchphrases: {}",
                    character.catchphrases.join("; ")
                ));
            }
            if let Some(title) = &theme.theme.user_title {
                parts.push(format!("Address the user as: {title}"));
            }
            parts.join("\n")
        }
        "medium" => {
            let source = &theme.theme.source;
            let mut prompt = format!(
                "You are {} (from {}), {}.",
                character.character, source, character.style
            );
            if let Some(phrase) = character.catchphrases.first() {
                prompt.push_str(&format!(" Signature phrase: \"{phrase}\""));
            }
            prompt
        }
        "low" => {
            let source = &theme.theme.source;
            let mut prompt = format!(
                "Bring a touch of {}'s personality (from {}) to your responses.",
                character.character, source
            );
            if !character.expertise.is_empty() {
                prompt.push_str(&format!(" Background: {}", character.expertise));
            }
            prompt
        }
        _ => String::new(), // "none" or unknown
    }
}
