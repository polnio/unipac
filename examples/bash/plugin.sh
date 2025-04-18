#!/usr/bin/env bash

ID="example"
NAME="My bash example plugin"

unipac_list_packages() {
  echo "Package Name,Id,Version,Description"
  echo "Package Example plugin,example,1.2.3,This is an example plugin"
  for i in {1..10}; do
    sleep 0.5
    echo "Progress $((i*10))"
  done
}

unipac_search() {
  list_packages
}

unipac_info() {
  list_packages
}

unipac_pre_install() {
  echo "Package Id,Version"
  echo "Package example,1.2.3"
  echo "Progress 100"
}

unipac_install() {
  echo "Progress 100"
}

unipac_remove() {
  echo "Progress 100"
}

source unipac-run
