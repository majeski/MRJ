#ifndef EXPR__H
#define EXPR__H

#include <stdint.h>

#include "many.h"

struct expr_t {
  int32_t type;
  void *e;
};

extern const int EXPR_TYPE_BINOP;
extern const int EXPR_TYPE_CALL;
extern const int EXPR_TYPE_FIELD;
extern const int EXPR_TYPE_LIT;
extern const int EXPR_TYPE_LIT_BOOL;
extern const int EXPR_TYPE_LIT_INT;
extern const int EXPR_TYPE_LIT_NULL;
extern const int EXPR_TYPE_LIT_STR;
extern const int EXPR_TYPE_NEW;
extern const int EXPR_TYPE_UNARY;

struct expr_binop_t {
  struct expr_t *lhs;
  struct expr_t *rhs;
  char *op;  // won't be freed
};

struct expr_unary_t {
  struct expr_t *e;
  char op;
};

struct expr_call_t {
  struct field_get_t *func;
  struct many_t *args;  // expr_t
};

struct expr_lit_t {
  int32_t type;
  char *lit; // class identifier for null literal
};

struct expr_new_t {
  char *type;
  struct expr_t *size; // nullable for objects, !null for arrays
};

extern struct expr_t *expr_binop_create(struct expr_t *lhs, struct expr_t *rhs,
                                        char *op);
extern struct expr_t *expr_unary_create(char op, struct expr_t *e);
extern struct expr_t *expr_call_create(struct field_get_t *field,
                                       struct many_t *args);
extern struct expr_t *expr_field_get_create(struct field_get_t *field);
extern struct expr_t *expr_lit_create(int32_t type, char *lit);
extern struct expr_t *expr_new_create(char *type, struct expr_t *size);

extern void expr_free(void *e);

#endif
