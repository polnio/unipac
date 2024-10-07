#include "unipac-shared.h"
#include <stdlib.h>
#include <string.h>

#define N_INSTALLED_PLUGIN 1
static char *INSTALLED_PLUGIN[N_INSTALLED_PLUGIN] = {"example-c"};

ListPackagesResult_t ffi_list_packages() {
  char **installed_plugin = malloc(sizeof(char *) * N_INSTALLED_PLUGIN);
  installed_plugin[0] = malloc(sizeof(char) * 15);
  /* installed_plugin[0] = "example-malloc"; */
  memcpy(installed_plugin[0], "example-malloc", 15);
  slice_boxed_char_ptr_t data = {.ptr = installed_plugin,
                                 .len = N_INSTALLED_PLUGIN};
  ListPackagesResult_t result = {.err = NULL, .data = data};
  return result;
}
