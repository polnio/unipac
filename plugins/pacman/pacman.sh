#!/usr/bin/env bash

case "$1" in
  get_id) echo "Response pacman" ;;
  get_name) echo "Response Pacman" ;;
  list_packages)
    echo "Package name,version"
    pacman -Q | tr ' ' , | sed 's/^/Package /'
    echo "Progress 100"
    ;;
  *) echo "Error Unknown command \"$1\"" ;;
esac
