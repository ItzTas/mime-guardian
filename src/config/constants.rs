pub const COMMAND: &str = "config";

pub const DIR: &str = "mime-guardian";

pub const FILE_NAME: &str = "config.toml";

pub const DEFAULTS_TABLE: &str = "defaults";

pub const GUARD_TABLE: &str = "guard";

pub const NOTIFY_KEY: &str = "notify";

pub const ADDED_ASSOCIATIONS_KEY: &str = "added_associations";

pub const WILDCARD: &str = "*";

pub const MEDIA_SEPARATOR: char = '/';

pub const NOTIFY_ENV: &str = "MMG_NOTIFY";

pub const SWITCH_WORDS: [(&str, bool); 8] = [
    ("on", true),
    ("off", false),
    ("true", true),
    ("false", false),
    ("1", true),
    ("0", false),
    ("yes", true),
    ("no", false),
];

pub const SWITCH_EXPECTED: &str = "on, off, true, false, 1, 0, yes or no";

pub const DEFAULT_SUMMARY: &str = "Default applications put back";

pub const DEFAULT_LINE: &str = "{mime}: {from} → {to}";

pub const DEFAULT_ICON: &str = "mime-guardian";
