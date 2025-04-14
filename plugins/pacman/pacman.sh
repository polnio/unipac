#!/usr/bin/env bash

DEPENDENCIES=(pacman)

ID="pacman"
NAME="Pacman"

list_packages() {
  echo "Package Name,Version"
  pacman -Q | tr ' ' , | sed 's/^/Package /'
  echo "Progress 100"
}

search() {
  echo "Package Registry,Name,Version,Description"
  pacman -Ss "$1" | while read -r line1 && read -r description; do
    echo -n "Package "
    echo -n "$line1" | tr / , | tr ' ' ,
    echo ",$description"
  done
  echo "Progress 100"
}

info() {
  echo "Package Registry,Name,Version,Description"
  echo -n "Package "
  set -o pipefail
  pacman -Si "$1" | cut -d: -f2 | cut -c2- | head -n4 | paste -sd , -
  if [ "$?" -eq 1 ]; then
    echo "Error Package $1 not found"
    exit 1
  fi
  echo "Progress 100"
}

source unipac-run
