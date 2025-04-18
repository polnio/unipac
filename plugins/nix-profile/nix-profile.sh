#!/usr/bin/env bash

DEPENDENCIES=(nix nix-search jq)

ID="nix-profile"
NAME="Nix (Profile)"

unipac_list_packages() {
  echo "Package Name,Version"
  nix profile list --json | jq -r '.elements | [ keys[] as $k | "Package " + $k + "," + (.[$k].storePaths[0] | capture("-\($k)-(?<version>.+)$").version)] | .[]'
  echo "Progress 100"
}

unipac_search() {
  echo "Package Name,Version,Description"
  nix-search -j "$1" | jq -r '"Package \(.package_attr_name),\(.package_pversion),\(.package_description)"'
  echo "Progress 100"
}

unipac_info() {
  echo "Package Name,Version,Description"
  nix-search -jm1 -n "$1" | jq -r '"Package \(.package_attr_name),\(.package_pversion),\(.package_description)"'
  echo "Progress 100"
}

unipac_pre_install() {
  echo "Package Id,Version"
  nix-search -jm1 -n "$1" | jq -r '"Package \(.package_attr_name),\(.package_pversion)"'
  echo "Progress 100"
}

unipac_install() {
  nix profile install "nixpkgs#$1"
  echo "Progress 100"
}

source unipac-run
