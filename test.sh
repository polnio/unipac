#!/usr/bin/env bash

tmp_dir=$(mktemp -d)
echo $tmp_dir
# trap 'rm -rf "$tmp_dir"' EXIT
cp -r $HOME/.nix-profile "$tmp_dir/profile"
ln -s "$tmp_dir/profile" "$tmp_dir/profile-link"
nix profile upgrade --all --profile "$tmp_dir/profile-link"
diff "$tmp_dir/profile/manifest.json" "$HOME/.nix-profile/manifest.json"
