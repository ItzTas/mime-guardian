# mime-guardian

Keeps your default applications the way you set them.

Linux desktops decide which program opens a link, a PDF or a folder by
reading `mimeapps.list` files. Any installed program can rewrite them, and
browsers and file managers often do it without asking. mime-guardian is meant
to watch those files, put your choice back when something changes it, and
tell you which type moved and to what. The command is `mmg`.

The defaults you want go in `~/.config/mime-guardian/config.toml`:

```toml
[defaults]
"text/html" = "firefox.desktop"
"x-scheme-handler/https" = "firefox.desktop"
"inode/directory" = "org.kde.dolphin.desktop"
"image/*" = "org.gnome.Loupe.desktop"
```

`image/*` expands to every image type in the shared MIME database, and an
exact type always beats a wildcard.

## Status

Early work. Only `mmg completions <shell>` and `mmg man` exist so far;
reading the config, `status`, `apply` and `watch` come next.

## Building

```
cargo build --release
```

The build script points `core.hooksPath` at `.githooks`, which checks commit
messages against Conventional Commits and runs clippy and the tests before a
push.

## License

MIT, see [LICENSE](LICENSE).
