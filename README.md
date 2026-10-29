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
| `notify` | `true` | Show a desktop notification when the watcher puts a default back. |
| `added_associations` | `true` | Also move your application to the front of `[Added Associations]` wherever the type already has an entry there. |
| `desktops` | `[]` | Desktop names to use when `XDG_CURRENT_DESKTOP` is empty, which can happen in a systemd user service. |

`mmg set text/html firefox` and `mmg unset text/html` edit the file for you and
keep its comments.

## Commands

| Command | What it does |
| --- | --- |
| `mmg init` | Create the configuration from the current defaults. |
| `mmg status` | List the protected types whose default is not the one you set. |
| `mmg apply [-n]` | Pin every protected type once. `-n` only shows what would change. |
| `mmg watch` | Keep running and put a type back as soon as something changes it. |
| `mmg set <mime> <app>`, `mmg unset <mime>` | Start or stop protecting a type. |
| `mmg log [-a]` | List the times an application took a type and mmg put it back. |
| `mmg restore [ID] [-l]` | Put back the files an earlier write replaced. |
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

## Building

```
cargo build --release
```

The build script points `core.hooksPath` at `.githooks`, which checks commit
messages against Conventional Commits and runs clippy and the tests before a
push.

## License

MIT, see [LICENSE](LICENSE).
