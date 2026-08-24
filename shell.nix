{pkgs ? import <nixpkgs> {}}:
let
  scriptsDir = ./private/scripts;
  scriptFiles =
    if builtins.pathExists scriptsDir
    then builtins.attrNames (builtins.readDir scriptsDir)
    else [];
  nixScripts = builtins.filter (f: pkgs.lib.hasSuffix ".nix" f) scriptFiles;
  privateScripts = map (f: import (scriptsDir + "/${f}") {inherit pkgs;}) nixScripts;
in
  pkgs.mkShell {
    nativeBuildInputs = with pkgs; [
    rustup
    nodejs
    gcc
    watchexec
  ]
  ++ privateScripts;
  shellHook = ''
    git config set core.hooksPath githooks

    rustup default stable
    rustup component add rust-src
    rustup component add rustfmt
    rustup component add clippy
    rustup target add x86_64-unknown-linux-gnu
  '';
  GIT_COMMIT_MSG_SCOPES = "lib cli devenv docs web misc";
}
