#!/usr/bin/env bash


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

source unipac-run
