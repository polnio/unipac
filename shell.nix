(import (
  let
    lock = builtins.fromJSON (builtins.readFile ./flake.lock);
    flake-compat-lock = lock.nodes.flake-compat.locked;
  in
  fetchTarball {
    url =
      flake-compat-lock.url
        or "https://github.com/edolstra/flake-compat/archive/${flake-compat-lock.rev}.tar.gz";
    sha256 = flake-compat-lock.narHash;
  }
) { src = ./.; }).shellNix
