#!/usr/bin/env bash

ID="example"
NAME="My bash example plugin"

list_packages() {
  echo "Package Name,Id,Version,Description"
  echo "Package Example plugin,example,1.2.3,This is an example plugin"
  for i in {1..10}; do
    sleep 0.5
    echo "Progress $((i*10))"
  done
}

search() {
  list_packages
}

info() {
  list_packages
}

source unipac-run
