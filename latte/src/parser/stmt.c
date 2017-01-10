#include "common.h"

const int32_t STMT_TYPE_ASSIGN = 0;
const int32_t STMT_TYPE_BLOCK = 1;
const int32_t STMT_TYPE_EMPTY = 2;
const int32_t STMT_TYPE_EXPR = 3;
const int32_t STMT_TYPE_FOR = 4;
const int32_t STMT_TYPE_IF = 5;
const int32_t STMT_TYPE_POSTFIX = 6;
const int32_t STMT_TYPE_RETURN = 7;
const int32_t STMT_TYPE_VAR_INIT = 8;
const int32_t STMT_TYPE_WHILE = 9;

struct stmt_t *stmt_create(int32_t type, void *s);

struct stmt_t *stmt_empty_create() {
  return stmt_create(STMT_TYPE_EMPTY, NULL);
}

struct stmt_t *stmt_var_decls_create(char *type, struct many_t *decls) {
  struct stmt_var_decls_t *s = malloc(sizeof(struct stmt_var_decls_t));
  CHECK_NULL(s);
  s->type = type;
  s->decls = decls;
  return stmt_create(STMT_TYPE_VAR_INIT, s);
}

struct var_decl_t *var_decl_create(char *ident, struct expr_t *e) {
  struct var_decl_t *d = malloc(sizeof(struct var_decl_t));
  CHECK_NULL(d);
  d->ident = ident;
  d->e = e;
  return d;
}

struct stmt_t *stmt_assign_create(struct field_get_t *field, struct expr_t *e) {
  struct stmt_assign_t *s = malloc(sizeof(struct stmt_assign_t));
  CHECK_NULL(s);
  s->field = field;
  s->e = e;
  return stmt_create(STMT_TYPE_ASSIGN, s);
}

struct stmt_t *stmt_postfix_create(struct field_get_t *field, int is_decr) {
  struct stmt_postfix_t *s = malloc(sizeof(struct stmt_postfix_t));
  CHECK_NULL(s);
  s->field = field;
  s->is_decr = is_decr;
  return stmt_create(STMT_TYPE_POSTFIX, s);
}

struct stmt_t *stmt_return_create(struct expr_t *e) {
  return stmt_create(STMT_TYPE_RETURN, e);
}

struct stmt_t *stmt_block_create(struct many_t *stmts) {
  return stmt_create(STMT_TYPE_BLOCK, stmts);
}

struct stmt_t *stmt_expr_create(struct expr_t *e) {
  return stmt_create(STMT_TYPE_EXPR, e);
}

struct stmt_t *stmt_if_create(struct expr_t *cond, struct stmt_t *if_s,
                              struct stmt_t *else_s) {
  struct stmt_if_t *s = malloc(sizeof(struct stmt_if_t));
  CHECK_NULL(s);
  s->cond = cond;
  s->if_s = if_s;
  s->else_s = else_s;
  return stmt_create(STMT_TYPE_IF, s);
}

struct stmt_t *stmt_while_create(struct expr_t *cond, struct stmt_t *stmts) {
  struct stmt_while_t *s = malloc(sizeof(struct stmt_while_t));
  CHECK_NULL(s);
  s->cond = cond;
  s->s = stmts;
  return stmt_create(STMT_TYPE_WHILE, s);
}

struct stmt_t *stmt_for_create(char *type, char *ident, struct expr_t *e,
                               struct stmt_t *stmt) {
  struct stmt_for_t *s = malloc(sizeof(struct stmt_for_t));
  CHECK_NULL(s);
  s->type = type;
  s->ident = ident;
  s->e = e;
  s->s = stmt;
  return stmt_create(STMT_TYPE_FOR, s);
}

struct stmt_t *stmt_create(int32_t type, void *s) {
  struct stmt_t *stmt = malloc(sizeof(struct stmt_t));
  CHECK_NULL(stmt);
  stmt->type = type;
  stmt->s = s;
  return stmt;
}

void stmt_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  int32_t type = ((struct stmt_t *)ptr)->type;
  void *s = ((struct stmt_t *)ptr)->s;
  if (s == NULL) {
    goto end;
  }

  if (type == STMT_TYPE_EMPTY) {
    // nothing to free
  } else if (type == STMT_TYPE_VAR_INIT) {
    struct stmt_var_decls_t *stmt = (struct stmt_var_decls_t *)s;
    free(stmt->type);
    many_free(stmt->decls, var_decl_free);
  } else if (type == STMT_TYPE_ASSIGN) {
    struct stmt_assign_t *stmt = (struct stmt_assign_t *)s;
    field_get_free(stmt->field);
  } else if (type == STMT_TYPE_POSTFIX) {
    struct stmt_postfix_t *stmt = (struct stmt_postfix_t *)s;
    field_get_free(stmt->field);
  } else if (type == STMT_TYPE_RETURN) {
    expr_free(s);
  } else if (type == STMT_TYPE_BLOCK) {
    many_free(s, stmt_free);
  } else if (type == STMT_TYPE_EXPR) {
    expr_free(s);
  } else if (type == STMT_TYPE_IF) {
    struct stmt_if_t *stmt = (struct stmt_if_t *)s;
    expr_free(stmt->cond);
    stmt_free(stmt->if_s);
    stmt_free(stmt->else_s);
  } else if (type == STMT_TYPE_WHILE) {
    struct stmt_while_t *stmt = (struct stmt_while_t *)s;
    expr_free(stmt->cond);
    stmt_free(stmt->s);
  } else if (type == STMT_TYPE_FOR) {
    struct stmt_for_t *stmt = (struct stmt_for_t *)s;
    free(stmt->type);
    free(stmt->ident);
    expr_free(stmt->e);
    stmt_free(stmt->s);
  } else {
    assert(0);
    exit(-1);
  }

end:
  free(ptr);
}

void var_decl_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct var_decl_t *d = (struct var_decl_t *)ptr;
  free(d->ident);
  expr_free(d->e);
  free(ptr);
}
