#include "common.h"

struct field_get_t *field_get_create(char *ident, struct field_get_t *field) {
  struct field_get_t *fg = malloc(sizeof(struct field_get_t));
  CHECK_NULL(fg);
  fg->ident = ident;
  fg->field = field;
  return fg;
}

void field_get_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct field_get_t *fg = ptr;
  free(fg->ident);
  field_get_free(fg->field);
  free(ptr);
}
