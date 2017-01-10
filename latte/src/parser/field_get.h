#ifndef FIELD_GET__H
#define FIELD_GET__H

#include <stdint.h>

#include "expr.h"

extern const int32_t FIELD_GET_TYPE_IDX;
extern const int32_t FIELD_GET_TYPE_STD;

struct field_get_t {
  int32_t type;
  struct expr_t *e;  // nullable
  void *ptr;         // char *field or expr_t *idx
};

extern void field_get_free(void *fg);

extern struct field_get_t *field_get_create(struct expr_t *e, char *field);
extern struct field_get_t *field_get_idx_create(struct expr_t *e,
                                                struct expr_t *idx);

#endif
