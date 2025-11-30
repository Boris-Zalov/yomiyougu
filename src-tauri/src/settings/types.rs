//! Core types for settings values and metadata

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The actual value stored for a setting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SettingValue {
    Bool(bool),
    String(String),
    Number(i64),
    Float(f64),
}

impl SettingValue {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            SettingValue::Bool(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            SettingValue::String(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<i64> {
        match self {
            SettingValue::Number(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            SettingValue::Float(v) => Some(*v),
            SettingValue::Number(v) => Some(*v as f64),
            _ => None,
        }
    }
}

impl From<bool> for SettingValue {
    fn from(v: bool) -> Self {
        SettingValue::Bool(v)
    }
}

impl From<String> for SettingValue {
    fn from(v: String) -> Self {
        SettingValue::String(v)
    }
}

impl From<&str> for SettingValue {
    fn from(v: &str) -> Self {
        SettingValue::String(v.to_string())
    }
}

impl From<i64> for SettingValue {
    fn from(v: i64) -> Self {
        SettingValue::Number(v)
    }
}

impl From<f64> for SettingValue {
    fn from(v: f64) -> Self {
        SettingValue::Float(v)
    }
}

impl From<SettingValue> for Value {
    fn from(v: SettingValue) -> Self {
        match v {
            SettingValue::Bool(b) => Value::Bool(b),
            SettingValue::String(s) => Value::String(s),
            SettingValue::Number(n) => Value::Number(n.into()),
            SettingValue::Float(f) => serde_json::Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
        }
    }
}

/// Widget type for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum WidgetType {
    /// On/off toggle switch
    Toggle,
    /// Text input field
    Input,
    /// Dropdown/radio selection with predefined options
    Select { options: Vec<SelectOption> },
    /// Numeric slider with min/max/step
    Slider { min: f64, max: f64, step: f64 },
    /// Color picker
    Color,
}

/// Option for select widgets with value and display label
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
        }
    }

    pub fn with_description(
        value: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: Some(description.into()),
        }
    }
}

/// A single setting definition with metadata for UI rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingItem {
    /// Unique identifier (e.g., "reading.direction")
    pub key: String,
    /// Display label
    pub label: String,
    /// Detailed description/help text
    pub description: String,
    /// Widget type for rendering
    pub widget: WidgetType,
    /// Current value
    pub value: SettingValue,
    /// Default value (for reset functionality)
    pub default_value: SettingValue,
    /// Whether this setting requires app restart
    #[serde(default)]
    pub requires_restart: bool,
    /// Platform availability (empty = all platforms)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub platforms: Vec<String>,
}

impl SettingItem {
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
        widget: WidgetType,
        default_value: SettingValue,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            description: description.into(),
            widget,
            value: default_value.clone(),
            default_value,
            requires_restart: false,
            platforms: Vec::new(),
        }
    }

    pub fn with_restart(mut self) -> Self {
        self.requires_restart = true;
        self
    }

    pub fn for_platforms(mut self, platforms: Vec<&str>) -> Self {
        self.platforms = platforms.into_iter().map(String::from).collect();
        self
    }
}

/// Category grouping related settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingCategory {
    /// Category identifier
    pub id: String,
    /// Display name
    pub label: String,
    /// Category description
    pub description: String,
    /// Icon name (for UI rendering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Settings in this category
    pub settings: Vec<SettingItem>,
}

impl SettingCategory {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: description.into(),
            icon: None,
            settings: Vec::new(),
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn add_setting(mut self, setting: SettingItem) -> Self {
        self.settings.push(setting);
        self
    }

    pub fn add_settings(mut self, settings: Vec<SettingItem>) -> Self {
        self.settings.extend(settings);
        self
    }
}

/// Complete settings structure stored as JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Schema version for migrations
    pub version: u32,
    /// Whether user completed initial setup
    pub setup_completed: bool,
    /// License/terms acceptance
    pub accepted_license: bool,
    /// All setting categories
    pub categories: Vec<SettingCategory>,
}

impl AppSettings {
    /// Get a setting value by key (format: "category.key")
    pub fn get(&self, key: &str) -> Option<&SettingValue> {
        for category in &self.categories {
            for setting in &category.settings {
                if setting.key == key {
                    return Some(&setting.value);
                }
            }
        }
        None
    }

    /// Set a setting value by key
    pub fn set(&mut self, key: &str, value: SettingValue) -> bool {
        for category in &mut self.categories {
            for setting in &mut category.settings {
                if setting.key == key {
                    setting.value = value;
                    return true;
                }
            }
        }
        false
    }

    /// Get all settings as a flat key-value map
    pub fn to_flat_map(&self) -> std::collections::HashMap<String, SettingValue> {
        let mut map = std::collections::HashMap::new();
        for category in &self.categories {
            for setting in &category.settings {
                map.insert(setting.key.clone(), setting.value.clone());
            }
        }
        map
    }

    /// Reset a setting to its default value
    pub fn reset(&mut self, key: &str) -> bool {
        for category in &mut self.categories {
            for setting in &mut category.settings {
                if setting.key == key {
                    setting.value = setting.default_value.clone();
                    return true;
                }
            }
        }
        false
    }

    /// Reset all settings to defaults
    pub fn reset_all(&mut self) {
        for category in &mut self.categories {
            for setting in &mut category.settings {
                setting.value = setting.default_value.clone();
            }
        }
    }
}
