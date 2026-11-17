# mime-guardian

Keeps your default applications the way you set them.

Linux desktops pick the program that opens a link, a PDF or a folder by
reading `mimeapps.list` files. Any installed program can rewrite those files,
and browsers and file managers often do it without asking. mime-guardian
watches them, puts your choice back when something changes it, and tells you
which type moved and to what. The command is `mmg`.

## Install

```
paru -S mime-guardian                              # Arch Linux (AUR); also mime-guardian-bin
nix profile install github:ItzTas/mime-guardian    # Nix
cargo install mime-guardian                        # crates.io
```

Each GitHub release also has `.deb` packages for amd64 and arm64, and prebuilt
tarballs.

Notifications go through `notify-send`, from libnotify. Wildcard types need
shared-mime-info, which most desktops already have.

## Getting started

```
mmg init                  # write the configuration from the defaults you use today
$EDITOR ~/.config/mime-guardian/config.toml
mmg status                # what differs from the configuration, and where
mmg apply                 # pin it once
mmg service install       # keep watching from now on, as a systemd user unit
```

`mmg init` reads the defaults your session uses now, so the first
configuration already matches your machine. Trim it down to the types you care
about. mmg leaves every other type alone.

## Configuration

`~/.config/mime-guardian/config.toml`:

```toml
[defaults]
"text/html" = "firefox.desktop"
"x-scheme-handler/https" = "firefox.desktop"
"inode/directory" = "org.kde.dolphin.desktop"
"image/*" = "org.gnome.Loupe.desktop"
"image/svg+xml" = "org.inkscape.Inkscape.desktop"

[guard]
notify = true
added_associations = true
desktops = []
```

`image/*` expands to every image type the shared MIME database knows, plus the
ones installed applications declare. An exact type always beats a wildcard, so
here SVG files open in Inkscape and every other image opens in Loupe. Old names
the database lists as aliases, such as `application/x-pdf`, count as the type
they point to.

If an application is not installed, mmg skips its types with a warning instead
of writing a default that opens nothing.

The `[guard]` table is optional:

| Key | Default | What it does |
| --- | --- | --- |
| `notify` | `true` | Show a desktop notification when the watcher puts a default back. See [Notifications](#notifications). |
| `added_associations` | `true` | Also move your application to the front of `[Added Associations]` wherever the type already has an entry there. |
| `desktops` | `[]` | Desktop names to use when `XDG_CURRENT_DESKTOP` is empty, which can happen in a systemd user service. |

`mmg set text/html firefox` and `mmg unset text/html` edit the file for you and
keep its comments.

### Notifications

`mmg config notify off` turns the notification off and `mmg config notify on`
turns it back on. It also accepts `true`/`false`, `1`/`0` and `yes`/`no`, and
prints the current state when you leave the value out. A running watcher reads
the change on its next pass, so you don't have to restart it.

The `MMG_NOTIFY` environment variable takes the same words and wins over the
configuration for an `mmg watch` started with it. `--no-notify` wins over both.
The systemd unit doesn't see what you export in your shell, so for the service
use `mmg config notify`, or run
`systemctl --user set-environment MMG_NOTIFY=0` and restart the unit.

The optional `[notification]` table changes what the notification says and how
it is sent:

```toml
[notification]
summary = "{count} defaults put back"
line = "{mime}: {from} → {to}"
icon = "preferences-desktop-default-applications"
transient = true
command = 'notify-send -u low "$MMG_SUMMARY" "$MMG_BODY"'
```

| Key | Default | What it does |
| --- | --- | --- |
| `summary` | `Default applications put back` | The title. `{count}` becomes the number of types put back. |
| `line` | `{mime}: {from} → {to}` | One body line per type. The body lists five types and counts the rest. |
| `icon` | `preferences-desktop-default-applications` | The icon name. Leave it empty for no icon. |
| `transient` | `false` | Ask the notification server not to keep the notification in its history. |
| `command` | none | A shell command to run instead of `notify-send`. |

`command` runs through `sh -c` and finds the message in its environment:
`MMG_SUMMARY`, `MMG_BODY`, `MMG_COUNT`, `MMG_ICON` and `MMG_TRANSIENT` (`true`
or `false`). mmg never pastes these values into the command text, so quote them
as you would any shell variable. If the command fails, `mmg watch` prints a
warning.

## Commands

| Command | What it does |
| --- | --- |
| `mmg init` | Create the configuration from the current defaults. |
| `mmg status` | List the protected types whose default is not the one you set. |
| `mmg apply [-n]` | Pin every protected type once. `-n` only shows what would change. |
| `mmg watch` | Keep running and put a type back as soon as something changes it. |
| `mmg set <mime> <app>`, `mmg unset <mime>` | Start or stop protecting a type. |
| `mmg config notify [on\|off]` | Show or switch the notification. See [Notifications](#notifications). |
| `mmg log [-a]` | List the times an application took a type and mmg put it back. |
| `mmg restore [ID] [-l]` | Put back the files an earlier write replaced. |
| `mmg adopt <command> [args]` | Run a tool such as handlr or `xdg-mime` and keep the defaults it changes. See [Changing defaults with other tools](#changing-defaults-with-other-tools). |
| `mmg service install\|uninstall\|status` | Run the watcher as a systemd user service. |
| `mmg completions <shell>`, `mmg man` | Print a completion script or the manual page. |

## How it works

The desktop specification lets several `mimeapps.list` files answer for the
same type, read in a fixed order. The per-desktop file in `~/.config`
(`kde-mimeapps.list`, `gnome-mimeapps.list` and so on) comes first, then
`~/.config/mimeapps.list`, then the system copies under `/etc/xdg` and
`/usr/share/applications`. KDE and GNOME usually write the per-desktop file,
which is why fixing `mimeapps.list` alone often does nothing.

For each protected type, mmg works out which application wins across that
whole chain. It only edits the files in `~/.config`: it writes the default into
`mimeapps.list`, rewrites any per-desktop file that holds a different default
for the type, and takes your application out of `[Removed Associations]`.
Everything else in those files stays byte for byte as it was, comments and
unknown sections included. A `mimeapps.list` that is a symlink, say from a
dotfiles manager, is written through the link.

Before each write, mmg saves the old files under
`~/.local/state/mime-guardian/backups`, keeping the last 50 writes, and records
every change in `~/.local/state/mime-guardian/history.jsonl`, which `mmg log`
reads.

The watcher follows directories rather than files, because many programs
replace a file by renaming a new one over it. It waits for a burst of writes to
settle before reading anything. Its own writes don't start a loop, since the
next pass finds nothing left to change. It also follows `config.toml`, so an
edit there takes effect right away.

## Running it as a service

`mmg service install` writes `~/.config/systemd/user/mime-guardian.service`,
then enables and starts it. The packages also ship the unit in
`/usr/lib/systemd/user`, so `systemctl --user enable --now mime-guardian`
works too.

A user service only sees the environment your session exported to systemd. If
`XDG_CURRENT_DESKTOP` is missing there, mmg can't tell which
`<desktop>-mimeapps.list` to guard, and `mmg service status` warns about it.
Export the variable at login:

```
dbus-update-activation-environment --systemd XDG_CURRENT_DESKTOP
```

Or set `desktops = ["kde"]` under `[guard]`.

## Changing defaults with other tools

Tools like [handlr-regex](https://github.com/Anomalocaridid/handlr-regex) or
`xdg-mime` write your choices to the same `mimeapps.list` files. mmg can't
tell that apart from a program taking over a type, so it would put the old
default back. Run the tool through `mmg adopt` instead:

```
mmg adopt handlr set text/html firefox.desktop
mmg adopt xdg-mime default org.gnome.Loupe.desktop image/png
```

mmg runs the command as given, and the watcher waits while it runs. When the
command exits, mmg compares the default of each type across your
`mimeapps.list` files with what it was before, and copies every type that
changed into `config.toml`. If the tool unsets a type you protect, mmg drops it
from the configuration. A wildcard key the tool writes as is, such as
`image/*`, stays a wildcard in the configuration. If the command fails, mmg
keeps nothing.

To keep typing the tool's own name, wrap it in your shell:

```sh
handlr() { mmg adopt handlr "$@"; }
```

In fish: `function handlr; mmg adopt handlr $argv; end`.

## Building

```
cargo build --release
```

The build script points `core.hooksPath` at `.githooks`, which checks commit
messages against Conventional Commits and runs clippy and the tests before a
push.

## License

MIT, see [LICENSE](LICENSE).
