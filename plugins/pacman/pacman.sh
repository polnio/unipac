#!/usr/bin/env bash

DEPENDENCIES=(pacman)

ID="pacman"
NAME="Pacman"

unipac_list_packages() {
  echo "Package Name,Version"
  pacman -Q | tr ' ' , | sed 's/^/Package /'
  echo "Progress 100"
}

unipac_search() {
  echo "Package Registry,Name,Version,Description"
  pacman -Ss "$1" | while read -r line1 && read -r description; do
    echo -n "Package "
    echo -n "$line1" | tr / , | tr ' ' ,
    echo ",$description"
  done
  echo "Progress 100"
}

unipac_info() {
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

unipac_pre_install() {
  echo "Package Id,Version"
  set -o pipefail
  data=$(pacman -Si "$1" | cut -d: -f2 | cut -c2- | head -n3 | paste -sd , - | sed 's/,/\//')
  if [ "$?" -eq 0 ]; then
    echo "Package $data"
  fi
  echo "Progress 100"
}

unipac_install() {
  step=0
  total=0
  installed=0

  pacman -S --noconfirm "$1" 2>&1 | while read -r line; do
    if [[ -z "$line" ]]; then
      step=$((step + 1))
      continue
    fi

    if [[ $step -eq 1 && "$line" =~ ^Package\ \(([0-9]+)\) ]]; then
      total=${BASH_REMATCH[1]}
    fi

    if [[ $step -eq 4 && "$line" = installing\ * ]]; then
      installed=$((installed + 1))
      echo "Progress $(($installed * 100 / $total))"
    fi
  done
}

source unipac-run
