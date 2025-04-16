#!/usr/bin/env bash

case "$1" in
  unipac_get_id) echo "Response error_with_name" ;;
  unipac_get_name) echo "Response Error tester" ;;
  unipac_list_packages) echo "Error This is a test error" ;;
  *) echo "Error Unknown command \"$1\"" ;;
esac
