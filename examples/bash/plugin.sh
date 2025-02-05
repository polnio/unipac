case "$1" in
  get_id) echo "Response example" ;;
  get_name) echo "Response My bash example plugin" ;;
  list_packages)
    echo "Package name,id,version,description"
    echo "Package Example plugin,example,1.2.3,This is an example plugin"
    echo "Progress 100"
    ;;
  *) echo "Error Unknown command \"$1\"" ;;
esac
