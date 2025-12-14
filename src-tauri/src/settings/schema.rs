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
    .add_settings(vec![
        SettingItem::new(
            "appearance.theme",
            "Theme",
            "Choose between light, dark, or system-matched theme",
            WidgetType::Select {
                options: vec![
                    SelectOption::with_description("light", "Light", "Clean, bright appearance"),
                    SelectOption::with_description("dark", "Dark", "Easy on the eyes in low light"),
                    SelectOption::with_description(
                        "system",
                        "System",
                        "Match your device settings",
                    ),
                ],
            },
            SettingValue::String("system".to_string()),
        ),
        SettingItem::new(
            "appearance.animations",
            "Enable Animations",
            "Show smooth transitions and animations",
            WidgetType::Toggle,
            SettingValue::Bool(true),
        ),
    ])
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
                        SelectOption::with_description(
                            "ltr",
                            "Left to Right",
                            "Western comic style",
                        ),
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
                "reading.page_mode",
                "Page Display Mode",
                "How pages are displayed while reading",
                WidgetType::Select {
                    options: vec![
                        SelectOption::with_description(
                            "single",
                            "Single Page",
                            "One page at a time",
                        ),
                        SelectOption::with_description(
                            "double",
                            "Double Page",
                            "Two pages side by side (desktop)",
                        ),
                        SelectOption::with_description(
                            "auto",
                            "Automatic",
                            "Adapts based on screen size",
                        ),
                    ],
                },
                SettingValue::String("auto".to_string()),
            ),
            SettingItem::new(
                "reading.fit_mode",
                "Image Fit Mode",
                "How images are scaled to fit the screen",
                WidgetType::Select {
                    options: vec![
                        SelectOption::new("width", "Fit Width"),
                        SelectOption::new("height", "Fit Height"),
                        SelectOption::new("contain", "Fit Screen"),
                        SelectOption::new("original", "Original Size"),
                    ],
                },
                SettingValue::String("width".to_string()),
            ),
            SettingItem::new(
                "reading.background_color",
                "Reader Background",
                "Background color while reading",
                WidgetType::Select {
                    options: vec![
                        SelectOption::new("black", "Black"),
                        SelectOption::new("dark_gray", "Dark Gray"),
                        SelectOption::new("white", "White"),
                        SelectOption::new("sepia", "Sepia"),
                    ],
                },
                SettingValue::String("black".to_string()),
            ),
            SettingItem::new(
                "reading.keep_screen_on",
                "Keep Screen On",
                "Prevent screen from sleeping while reading",
                WidgetType::Toggle,
                SettingValue::Bool(true),
            )
            .for_platforms(vec!["android", "ios"]),
            SettingItem::new(
                "reading.tap_zones",
                "Tap Navigation Zones",
                "Enable tap zones for page navigation",
                WidgetType::Toggle,
                SettingValue::Bool(true),
            ),
            SettingItem::new(
                "reading.show_page_number",
                "Show Page Number",
                "Display current page number while reading",
                WidgetType::Toggle,
                SettingValue::Bool(true),
            ),
            SettingItem::new(
                "reading.preload_pages",
                "Preload Pages",
                "Number of pages to preload ahead",
                WidgetType::Slider {
                    min: 1.0,
                    max: 10.0,
                    step: 1.0,
                },
                SettingValue::Number(3),
            ),
        ])
}

fn create_library_category() -> SettingCategory {
    SettingCategory::new("library", "Library", "Organize and manage your collection")
        .with_icon("library")
        .add_settings(vec![
            SettingItem::new(
                "library.default_view",
                "Default View",
                "How your library is displayed",
                WidgetType::Select {
                    options: vec![
                        SelectOption::new("grid", "Grid"),
                        SelectOption::new("list", "List"),
                        SelectOption::new("compact", "Compact Grid"),
                    ],
                },
                SettingValue::String("grid".to_string()),
            ),
            SettingItem::new(
                "library.sort_by",
                "Sort By",
                "Default sorting for your library",
                WidgetType::Select {
                    options: vec![
                        SelectOption::new("title", "Title"),
                        SelectOption::new("last_read", "Last Read"),
                        SelectOption::new("date_added", "Date Added"),
                        SelectOption::new("unread", "Unread Count"),
                    ],
                },
                SettingValue::String("last_read".to_string()),
            ),
            SettingItem::new(
                "library.show_unread_badge",
                "Show Unread Badge",
                "Display unread chapter count on covers",
                WidgetType::Toggle,
                SettingValue::Bool(true),
            ),
            SettingItem::new(
                "library.cover_size",
                "Cover Size",
                "Size of cover images in library view",
                WidgetType::Select {
                    options: vec![
                        SelectOption::new("small", "Small"),
                        SelectOption::new("medium", "Medium"),
                        SelectOption::new("large", "Large"),
                    ],
                },
                SettingValue::String("medium".to_string()),
            ),
            SettingItem::new(
                "library.show_continue_reading",
                "Show Continue Reading",
                "Show recently read items for quick access",
                WidgetType::Toggle,
                SettingValue::Bool(true),
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
            "sync.google_drive_enabled",
            "Google Drive Sync",
            "Sync your library and progress to Google Drive",
            WidgetType::Toggle,
            SettingValue::Bool(false),
        ),
        SettingItem::new(
            "sync.sync_reading_progress",
            "Sync Reading Progress",
            "Keep your reading position in sync across devices",
            WidgetType::Toggle,
            SettingValue::Bool(true),
        ),
        SettingItem::new(
            "sync.sync_library",
            "Sync Library",
            "Sync your library organization and metadata",
            WidgetType::Toggle,
            SettingValue::Bool(true),
        ),
        SettingItem::new(
            "sync.auto_sync_interval",
            "Auto Sync Interval",
            "How often to automatically sync in the background",
            WidgetType::Select {
                options: vec![
                    SelectOption::new("manual", "Manual Only"),
                    SelectOption::new("15min", "Every 15 minutes"),
                    SelectOption::new("30min", "Every 30 minutes"),
                    SelectOption::new("1hour", "Every hour"),
                    SelectOption::new("daily", "Once a day"),
                ],
            },
            SettingValue::String("30min".to_string()),
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
    .add_settings(vec![
        SettingItem::new(
            "advanced.cache_size_mb",
            "Image Cache Size (MB)",
            "Maximum disk space for cached images",
            WidgetType::Slider {
                min: 100.0,
                max: 2000.0,
                step: 100.0,
            },
            SettingValue::Number(500),
        ),
        SettingItem::new(
            "advanced.enable_logging",
            "Enable Debug Logging",
            "Log detailed information for troubleshooting",
            WidgetType::Toggle,
            SettingValue::Bool(false),
        ),
        SettingItem::new(
            "advanced.image_quality",
            "Image Quality",
            "Quality vs performance tradeoff for image rendering",
            WidgetType::Select {
                options: vec![
                    SelectOption::new("low", "Low (Faster)"),
                    SelectOption::new("medium", "Medium"),
                    SelectOption::new("high", "High (Best Quality)"),
                ],
            },
            SettingValue::String("medium".to_string()),
        ),
    ])
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
