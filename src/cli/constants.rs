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

pub const MAN_ENVIRONMENT: [(&str, &str); 1] =
    [("RUST_LOG", "The log filter mmg reads when -v is not given.")];

pub const MAN_SEE_ALSO: &str = "xdg-mime(1), update-mime-database(1)";

pub const MAN_DOCUMENTATION: &str = concat!(
    "The full documentation lives at ",
    env!("CARGO_PKG_REPOSITORY")
);

pub const ROFF_PREAMBLE: &str = r#".ie \n(.g .ds Aq \(aq
.el .ds Aq '
"#;
