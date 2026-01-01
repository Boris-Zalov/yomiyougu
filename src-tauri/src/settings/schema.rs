//! Default settings schema with manga/comic-appropriate defaults

use super::types::*;

/// Current settings schema version (increment when making breaking changes)
pub const SETTINGS_VERSION: u32 = 1;

/// Create the default settings schema with all categories and settings
pub fn create_default_settings() -> AppSettings {
    AppSettings {
        version: SETTINGS_VERSION,
        setup_completed: false,
        accepted_license: false,
        updated_at: 0, // Will be set when first saved
        categories: vec![
            create_appearance_category(),
            create_reading_category(),
            create_library_category(),
            create_sync_category(),
            create_advanced_category(),
        ],
    }
}

fn create_appearance_category() -> SettingCategory {
    SettingCategory::new(
        "appearance",
        "Appearance",
        "Customize the look and feel of the application",
    )
    .with_icon("palette")
    .add_settings(vec![SettingItem::new(
        "appearance.theme",
        "Theme",
        "Choose between light, dark, or system-matched theme",
        WidgetType::Select {
            options: vec![
                SelectOption::with_description("light", "Light", "Clean, bright appearance"),
                SelectOption::with_description("dark", "Dark", "Easy on the eyes in low light"),
                SelectOption::with_description("system", "System", "Match your device settings"),
            ],
        },
        SettingValue::String("system".to_string()),
    )])
}

fn create_reading_category() -> SettingCategory {
    SettingCategory::new("reading", "Reading", "Configure your reading experience")
        .with_icon("book-open")
        .add_settings(vec![
            SettingItem::new(
                "reading.direction",
                "Reading Direction",
                "Default page turn direction for manga and comics",
                WidgetType::Select {
                    options: vec![
                        SelectOption::with_description(
                            "rtl",
                            "Right to Left",
                            "Traditional manga style (Japanese)",
                        ),
                        SelectOption::with_description("ltr", "Left to Right", "Western comic style"),
                        SelectOption::with_description(
                            "vertical",
                            "Vertical Scroll",
                            "Webtoon/manhwa style",
                        ),
                    ],
                },
                SettingValue::String("rtl".to_string()),
            ),
            SettingItem::new(
                "reading.page_display_mode",
                "Page Display Mode",
                "How pages are displayed in the reader",
                WidgetType::Select {
                    options: vec![
                        SelectOption::with_description("single", "Single Page", "One page at a time"),
                        SelectOption::with_description("double", "Double Page", "Two pages side by side"),
                        SelectOption::with_description("continuous", "Continuous", "Scroll through pages continuously"),
                    ],
                },
                SettingValue::String("single".to_string()),
            ),
            SettingItem::new(
                "reading.image_fit_mode",
                "Image Fit Mode",
                "How images are scaled to fit the screen",
                WidgetType::Select {
                    options: vec![
                        SelectOption::with_description("fit_width", "Fit Width", "Scale to screen width"),
                        SelectOption::with_description("fit_height", "Fit Height", "Scale to screen height"),
                        SelectOption::with_description("fit_screen", "Fit Screen", "Fit entire page on screen"),
                        SelectOption::with_description("original", "Original", "No scaling"),
                    ],
                },
                SettingValue::String("fit_width".to_string()),
            ),
        ])
}

fn create_library_category() -> SettingCategory {
    SettingCategory::new("library", "Library", "Organize and manage your collection")
        .with_icon("library")
        .add_settings(vec![
            SettingItem::new(
                "library.save_to_app_storage",
                "Save Imported Files to App Storage",
                "Store imported files in app storage for reliable access. Files remain available even if originals are moved or deleted. Note: This option is always enabled on Android due to system restrictions.",
                WidgetType::Toggle,
                SettingValue::Bool(false),
            ),
        ])
}

fn create_sync_category() -> SettingCategory {
    SettingCategory::new(
        "sync",
        "Cloud & Sync",
        "Backup and sync your data across devices",
    )
    .with_icon("cloud")
    .add_settings(vec![
        SettingItem::new(
            "sync.books",
            "Sync Comic Books",
            "Upload and sync your comic book files to Google Drive",
            WidgetType::Toggle,
            SettingValue::Bool(false),
        ),
        SettingItem::new(
            "sync.settings",
            "Sync Settings",
            "Sync your app settings across devices",
            WidgetType::Toggle,
            SettingValue::Bool(false),
        ),
        SettingItem::new(
            "sync.progress",
            "Sync Reading Progress",
            "Sync your reading progress and bookmarks across devices",
            WidgetType::Toggle,
            SettingValue::Bool(true),
        ),
    ])
}

fn create_advanced_category() -> SettingCategory {
    SettingCategory::new(
        "advanced",
        "Advanced",
        "Advanced settings and developer options",
    )
    .with_icon("cog")
    .add_settings(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_creation() {
        let settings = create_default_settings();
        assert_eq!(settings.version, SETTINGS_VERSION);
        assert!(!settings.setup_completed);
        assert!(!settings.categories.is_empty());
    }

    #[test]
    fn test_get_setting() {
        let settings = create_default_settings();
        let theme = settings.get("appearance.theme");
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().as_string(), Some("system"));
    }

    #[test]
    fn test_set_setting() {
        let mut settings = create_default_settings();
        let success = settings.set("appearance.theme", SettingValue::String("dark".to_string()));
        assert!(success);
        assert_eq!(
            settings.get("appearance.theme").unwrap().as_string(),
            Some("dark")
        );
    }
}
