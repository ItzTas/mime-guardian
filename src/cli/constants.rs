pub const DEFAULT_FILTER: &str = "warn";

pub const VERBOSE_FILTER: &str = "mime_guardian=debug";

pub const TRACE_FILTER: &str = "mime_guardian=trace";

pub const MAN_TITLE: &str = "MMG";

pub const MAN_MANUAL: &str = "User Commands";

pub const MAN_EMPTY_DATE: &str = " ";

pub const MAN_COMMAND_VALUE_NAME: &str = "COMMAND";

pub const MAN_HIDDEN_ARGS: [&str; 2] = ["help", "verbose"];

pub const MAN_SYNOPSIS_SECTION: &str = "SYNOPSIS";

pub const MAN_OPTIONS_SECTION: &str = "OPTIONS";

pub const MAN_COMMANDS_SECTION: &str = "COMMANDS";

pub const MAN_ENVIRONMENT_SECTION: &str = "ENVIRONMENT";

pub const MAN_SEE_ALSO_SECTION: &str = "SEE ALSO";

pub const MAN_ENVIRONMENT: [(&str, &str); 2] = [
    (
        "MMG_NOTIFY",
        "Turns the notifications of mmg watch on or off over guard.notify: on, off, true, false, 1, 0, yes or no.",
    ),
    ("RUST_LOG", "The log filter mmg reads when -v is not given."),
];

pub const MAN_SEE_ALSO: &str = "xdg-mime(1), update-mime-database(1)";

pub const MAN_DOCUMENTATION: &str = concat!(
    "The full documentation lives at ",
    env!("CARGO_PKG_REPOSITORY")
);

pub const ROFF_PREAMBLE: &str = r#".ie \n(.g .ds Aq \(aq
.el .ds Aq '
"#;

pub const UNSET: &str = "(none)";

pub const INIT_COMMAND: &str = "init";

pub const INIT_NEXT: &str =
    "(edit it to keep only the types you care about, then use \"mmg apply\" to pin them)";

pub const STATUS_OVERRIDDEN: &str = "Types another application took:";

pub const STATUS_OVERRIDDEN_HINT: &str = "(use \"mmg apply\" to put them back)";

pub const STATUS_UNPINNED: &str = "Types not pinned in your files yet:";

pub const STATUS_UNPINNED_HINT: &str = "(use \"mmg apply\" to write them)";

pub const STATUS_SKIPPED: &str = "Types skipped, their application is not installed:";

pub const STATUS_SKIPPED_HINT: &str = "(install the application or change it in the configuration)";

pub const STATUS_CLEAN: &str = "nothing to apply, every protected type is set";

pub const APPLY_PINNED: &str = "Pinned:";

pub const APPLY_DRY_RUN: &str = "Files mmg apply would write:";

pub const APPLY_CONFLICTS: &str =
    "these files changed while mmg was writing and were left alone; run mmg apply again:";

pub const LOG_DEFAULT_LIMIT: usize = 20;

pub const LOG_EMPTY: &str =
    "no application has taken a protected type yet (use --all to include first pins)";

pub const RESTORE_COMMAND: &str = "restore";

pub const RESTORE_EMPTY: &str = "no backups yet; mmg makes one every time it writes";

pub const RESTORE_WATCH_HINT: &str = "(a running mmg watch pins the configured defaults again; change the configuration to keep these)";

pub const DESKTOP_SUFFIX: &str = ".desktop";

pub const SET_COMMAND: &str = "set";

pub const UNSET_COMMAND: &str = "unset";

pub const SERVICE_COMMAND: &str = "service";

pub const SERVICE_NOT_INSTALLED: &str =
    "the systemd unit is not installed (use \"mmg service install\")";

pub const SERVICE_NO_DESKTOP: &str = "the systemd user session has no XDG_CURRENT_DESKTOP, so the unit only sees mimeapps.list; run \"dbus-update-activation-environment --systemd XDG_CURRENT_DESKTOP\" at login, or set guard.desktops in the configuration";

pub const ADOPT_COMMAND: &str = "adopt";

pub const ADOPT_KEPT: &str = "Defaults kept in";

pub const ADOPT_KEPT_HINT: &str = "(mmg now protects these defaults)";

pub const ADOPT_REFUSED: &str = "Changes mmg cannot protect:";

pub const ADOPT_NO_CONFIG: &str =
    "(no mmg configuration yet, so nothing was kept; use \"mmg init\" to create one)";

pub const CONFIG_COMMAND: &str = "config";

pub const SWITCH_ON: &str = "on";

pub const SWITCH_OFF: &str = "off";
