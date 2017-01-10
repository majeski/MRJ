#include "common.h"

const int EXPR_TYPE_BINOP = 0;
const int EXPR_TYPE_CALL = 1;
const int EXPR_TYPE_FIELD = 2;
const int EXPR_TYPE_LIT = 3;
const int EXPR_TYPE_LIT_BOOL = 4;
const int EXPR_TYPE_LIT_INT = 5;
const int EXPR_TYPE_LIT_NULL = 6;
const int EXPR_TYPE_LIT_STR = 7;
const int EXPR_TYPE_NEW_ARR = 8;
const int EXPR_TYPE_UNARY = 9;

struct expr_t *expr_create(int32_t type, void *e);

struct expr_t *expr_binop_create(struct expr_t *lhs, struct expr_t *rhs,
                                 char *op) {
  struct expr_binop_t *e = malloc(sizeof(struct expr_binop_t));
  CHECK_NULL(e);
  e->lhs = lhs;
  e->rhs = rhs;
  e->op = op;
  return expr_create(EXPR_TYPE_BINOP, e);
}

struct expr_t *expr_unary_create(char op, struct expr_t *expr) {
  struct expr_unary_t *e = malloc(sizeof(struct expr_unary_t));
  CHECK_NULL(e);
  e->e = expr;
  e->op = op;
  return expr_create(EXPR_TYPE_UNARY, e);
}

struct expr_t *expr_call_create(struct field_get_t *func, struct many_t *args) {
  struct expr_call_t *e = malloc(sizeof(struct expr_call_t));
  CHECK_NULL(e);
  e->func = func;
  e->args = args;
  return expr_create(EXPR_TYPE_CALL, e);
}

struct expr_t *expr_field_get_create(struct field_get_t *field) {
  return expr_create(EXPR_TYPE_FIELD, field);
}

struct expr_t *expr_lit_create(int32_t type, char *lit) {
  struct expr_lit_t *e = malloc(sizeof(struct expr_lit_t));
  CHECK_NULL(e);
  e->type = type;
  e->lit = lit;
  return expr_create(EXPR_TYPE_LIT, e);
}

struct expr_t *expr_new_array_create(char *type, struct expr_t *size) {
  struct expr_new_arr_t *e = malloc(sizeof(struct expr_new_arr_t));
  CHECK_NULL(e);
  e->type = type;
  e->size = size;
  return expr_create(EXPR_TYPE_NEW_ARR, e);
}

struct expr_t *expr_create(int32_t type, void *e) {
  struct expr_t *expr = malloc(sizeof(struct expr_t));
  CHECK_NULL(e);
  expr->type = type;
  expr->e = e;
  return expr;
}

void expr_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  int32_t type = ((struct expr_t *)ptr)->type;
  void *e = ((struct expr_t *)ptr)->e;
  if (e == NULL) {
    goto end;
  }

  if (type == EXPR_TYPE_BINOP) {
    struct expr_binop_t *expr = (struct expr_binop_t *)e;
    expr_free(expr->lhs);
    expr_free(expr->rhs);
  } else if (type == EXPR_TYPE_UNARY) {
    struct expr_unary_t *expr = (struct expr_unary_t *)e;
    expr_free(expr->e);
  } else if (type == EXPR_TYPE_CALL) {
    struct expr_call_t *expr = (struct expr_call_t *)e;
    field_get_free(expr->func);
    many_free(expr->args, expr_free);
  } else if (type == EXPR_TYPE_FIELD) {
    field_get_free(e);
  } else if (type == EXPR_TYPE_LIT) {
    struct expr_lit_t *expr = (struct expr_lit_t *)e;
    free(expr->lit);
  } else if (type == EXPR_TYPE_NEW_ARR) {
    struct expr_new_arr_t *expr = (struct expr_new_arr_t *)e;
    free(expr->type);
    expr_free(expr->size);
  } else {
    assert(0);
    exit(-1);
  }

end:
  free(ptr);
}
