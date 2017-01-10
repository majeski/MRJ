#include "common.h"

const int32_t FIELD_GET_TYPE_IDX = 0;
const int32_t FIELD_GET_TYPE_STD = 1;

struct field_get_t *field_get_create(struct expr_t *e, char *field) {
  struct field_get_t *fg = malloc(sizeof(struct field_get_t));
  CHECK_NULL(fg);
  fg->type = FIELD_GET_TYPE_STD;
  fg->e = e;
  fg->ptr = field;
  return fg;
}

struct field_get_t *field_get_idx_create(struct expr_t *e, struct expr_t *idx) {
  struct field_get_t *fg = malloc(sizeof(struct field_get_t));
  CHECK_NULL(fg);
  fg->type = FIELD_GET_TYPE_IDX;
  fg->e = e;
  fg->ptr = idx;
  return fg;
}

void field_get_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct field_get_t *fg = ptr;
  expr_free(fg->e);
  if (fg->type == FIELD_GET_TYPE_IDX) {
    expr_free(fg->ptr);
  } else {
    free(fg->ptr);
  }
  free(ptr);
}
