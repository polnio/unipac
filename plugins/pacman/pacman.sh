#!/usr/bin/env bash

get_id() {
  echo "Response pacman"
}

get_name() {
  echo "Response Pacman"
}

list_packages() {
  echo "Package Name,Version"
  pacman -Q | tr ' ' , | sed 's/^/Package /'
  echo "Progress 100"
}

source unipac-run
