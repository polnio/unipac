#include "unipac-shared.h"
#include <stdlib.h>
#include <string.h>

#define N_INSTALLED_PLUGINS 1
static char *INSTALLED_PLUGINS[N_INSTALLED_PLUGINS] = {"example-c"};

ListPackagesResult_t ffi_list_packages() {
  char **installed_plugins = malloc(sizeof(char *) * N_INSTALLED_PLUGINS);
  for (size_t i = 0; i < N_INSTALLED_PLUGINS; i++) {
    char *installed_plugin = INSTALLED_PLUGINS[i];
    size_t len = strlen(installed_plugin);
    installed_plugins[i] = malloc(sizeof(char) * len);
    memcpy(installed_plugins[i], installed_plugin, len);
  }

  slice_boxed_char_ptr_t data = {.ptr = installed_plugins,
                                 .len = N_INSTALLED_PLUGINS};
  ListPackagesResult_t result = {.err = NULL, .data = data};
  return result;
}
