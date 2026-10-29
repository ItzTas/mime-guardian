pub const COMMAND: &str = "xdg";

pub const HOME: &str = "HOME";

pub const CONFIG_HOME: &str = "XDG_CONFIG_HOME";

pub const CONFIG_DIRS: &str = "XDG_CONFIG_DIRS";

pub const DATA_HOME: &str = "XDG_DATA_HOME";

pub const DATA_DIRS: &str = "XDG_DATA_DIRS";

pub const STATE_HOME: &str = "XDG_STATE_HOME";

pub const CURRENT_DESKTOP: &str = "XDG_CURRENT_DESKTOP";

pub const CONFIG_HOME_DEFAULT: &str = ".config";

pub const DATA_HOME_DEFAULT: &str = ".local/share";

pub const STATE_HOME_DEFAULT: &str = ".local/state";

pub const CONFIG_DIRS_DEFAULT: &[&str] = &["/etc/xdg"];

pub const DATA_DIRS_DEFAULT: &[&str] = &["/usr/local/share", "/usr/share"];

pub const LIST_SEPARATOR: char = ':';
