pub const COMMAND: &str = "service";

pub const SYSTEMCTL: &str = "systemctl";

pub const USER_FLAG: &str = "--user";

pub const UNIT_NAME: &str = "mime-guardian.service";

pub const UNIT_DIR: &str = "systemd/user";

pub const DESKTOP_VARIABLE: &str = "XDG_CURRENT_DESKTOP=";

pub const UNIT_TEMPLATE: &str = "[Unit]
Description=Keep default applications the way you set them
Documentation={documentation}
PartOf=graphical-session.target
After=graphical-session.target

[Service]
ExecStart={program} watch
Restart=on-failure
RestartSec=5

[Install]
WantedBy=graphical-session.target
";
