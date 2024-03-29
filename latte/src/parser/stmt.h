#ifndef STMT__H
#define STMT__H

#include <stdint.h>

#include "many.h"

extern const int32_t STMT_TYPE_ASSIGN;
extern const int32_t STMT_TYPE_BLOCK;
extern const int32_t STMT_TYPE_EMPTY;
extern const int32_t STMT_TYPE_EXPR;
extern const int32_t STMT_TYPE_FOR;
extern const int32_t STMT_TYPE_IF;
extern const int32_t STMT_TYPE_POSTFIX;
extern const int32_t STMT_TYPE_RETURN;
extern const int32_t STMT_TYPE_VAR_INIT;
extern const int32_t STMT_TYPE_WHILE;

struct stmt_t {
  int32_t type;
  void *s;  // nullable (empty statement ";")
};

struct stmt_var_decls_t {
  char *type;
  struct many_t *decls;
};

struct var_decl_t {
  char *ident;
  struct expr_t *e;  // nullable (for no initial value in declaration)
};

struct stmt_assign_t {
  struct field_get_t *field;
  struct expr_t *e;
};

struct stmt_postfix_t {
  struct field_get_t *field;
  int32_t is_decr;
};

struct stmt_if_t {
  struct expr_t *cond;
  struct stmt_t *if_s;
  struct stmt_t *else_s;  // nullable
};

struct stmt_while_t {
  struct expr_t *cond;
  struct stmt_t *s;
};

struct stmt_for_t {
  char *type;
  char *ident;
  struct expr_t *e;
  struct stmt_t *s;
};

extern struct stmt_t *stmt_empty_create();
extern struct stmt_t *stmt_var_decls_create(char *type, struct many_t *decls);
extern struct var_decl_t *var_decl_create(char *ident, struct expr_t *e);

extern struct stmt_t *stmt_assign_create(struct field_get_t *field,
                                         struct expr_t *e);
extern struct stmt_t *stmt_postfix_create(struct field_get_t *field,
                                          int is_decr);
extern struct stmt_t *stmt_return_create(struct expr_t *e);
extern struct stmt_t *stmt_block_create(struct many_t *stmts);
extern struct stmt_t *stmt_expr_create(struct expr_t *e);
extern struct stmt_t *stmt_if_create(struct expr_t *cond, struct stmt_t *if_s,
                                     struct stmt_t *else_s);
extern struct stmt_t *stmt_while_create(struct expr_t *cond, struct stmt_t *s);
extern struct stmt_t *stmt_for_create(char *type, char *ident, struct expr_t *e,
                                      struct stmt_t *s);

extern void stmt_free(void *s);
extern void var_decl_free(void *d);

#endif
