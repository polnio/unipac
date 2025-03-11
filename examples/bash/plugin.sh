#!/usr/bin/env bash

get_id() {
  echo "Response example"
}

get_name() {
  echo "Response My bash example plugin"
}

list_packages() {
  echo "Package name,id,version,description"
  echo "Package Example plugin,example,1.2.3,This is an example plugin"
  for i in {1..10}; do
    sleep 0.5
    echo "Progress $((i*10))"
  done
}

source unipac-run
