#include "common.h"

const int EXPR_TYPE_BINOP = 1;
const int EXPR_TYPE_UNARY = 2;
const int EXPR_TYPE_CALL = 3;
const int EXPR_TYPE_IDENT = 4;
const int EXPR_TYPE_LIT = 5;
const int EXPR_TYPE_LIT_INT = 100;
const int EXPR_TYPE_LIT_STR = 101;
const int EXPR_TYPE_LIT_BOOL = 102;

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

struct expr_t *expr_call_create(char *fname, struct many_t *args) {
  struct expr_call_t *e = malloc(sizeof(struct expr_call_t));
  CHECK_NULL(e);
  e->fname = fname;
  e->args = args;
  return expr_create(EXPR_TYPE_CALL, e);
}

struct expr_t *expr_ident_create(char *ident) {
  return expr_create(EXPR_TYPE_IDENT, ident);
}

struct expr_t *expr_lit_create(int32_t type, char *lit) {
  struct expr_lit_t *e = malloc(sizeof(struct expr_lit_t));
  CHECK_NULL(e);
  e->type = type;
  e->lit = lit;
  return expr_create(EXPR_TYPE_LIT, e);
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
    free(expr->fname);
    many_free(expr->args, expr_free);
  } else if (type == EXPR_TYPE_IDENT) {
    free(e);
  } else if (type == EXPR_TYPE_LIT) {
    struct expr_lit_t *expr = (struct expr_lit_t *)e;
    free(expr->lit);
  } else {
    assert(0);
    exit(-1);
  }

end:
  free(ptr);
}
