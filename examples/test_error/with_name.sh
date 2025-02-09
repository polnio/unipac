#!/usr/bin/env bash

case "$1" in
  get_id) echo "Response error_with_name" ;;
  get_name) echo "Response Error tester" ;;
  list_packages) echo "Error This is a test error" ;;
  *) echo "Error Unknown command \"$1\"" ;;
esac
