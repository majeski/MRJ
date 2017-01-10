#include <stdlib.h>
#include <string.h>

#include "common.h"

char *array_type_create(char *type) {
  char *res = malloc(strlen(type) + 1);
  CHECK_NULL(res);
  strcpy(res + 1, type);
  res[0] = '[';
  return res;
}
