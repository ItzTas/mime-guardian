{
  lib,
  stdenv,
  rustPlatform,
  installShellFiles,
  makeWrapper,
  libnotify,
}:

let
  manifest = (lib.importTOML ../Cargo.toml).package;
  bin = "mmg";
in
rustPlatform.buildRustPackage {
  pname = manifest.name;
  version = manifest.version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../LICENSE
      ../build
      ../contrib
      ../src
      ../tests
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    installShellFiles
    makeWrapper
  ];

  doCheck = false;

  postInstall = ''
    install -Dm0644 contrib/${manifest.name}.service $out/lib/systemd/user/${manifest.name}.service
    substituteInPlace $out/lib/systemd/user/${manifest.name}.service \
      --replace-fail /usr/bin/${bin} $out/bin/${bin}
    install -Dm0644 contrib/${manifest.name}.desktop $out/share/applications/${manifest.name}.desktop
    install -Dm0644 contrib/${manifest.name}.svg $out/share/icons/hicolor/scalable/apps/${manifest.name}.svg

    wrapProgram $out/bin/${bin} --suffix PATH : ${lib.makeBinPath [ libnotify ]}
  ''
  + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    installShellCompletion --cmd ${bin} \
      --bash <($out/bin/${bin} completions bash) \
      --zsh <($out/bin/${bin} completions zsh) \
      --fish <($out/bin/${bin} completions fish)

    $out/bin/${bin} man >${bin}.1
    installManPage ${bin}.1
  '';

  meta = {
    description = manifest.description;
    homepage = manifest.repository;
    license = lib.licenses.mit;
    mainProgram = bin;
    platforms = lib.platforms.linux;
  };
}
