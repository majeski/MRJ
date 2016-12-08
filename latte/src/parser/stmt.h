#ifndef STMT__H
#define STMT__H

#include <stdint.h>

#include "many.h"

extern const int32_t STMT_TYPE_VAR_INIT;
extern const int32_t STMT_TYPE_ASSIGN;
extern const int32_t STMT_TYPE_POSTFIX;
extern const int32_t STMT_TYPE_RETURN;
extern const int32_t STMT_TYPE_BLOCK;
extern const int32_t STMT_TYPE_EXPR;
extern const int32_t STMT_TYPE_IF;
extern const int32_t STMT_TYPE_WHILE;

struct stmt_t {
  int32_t type;
  void *s;
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
  char *ident;
  struct expr_t *e;
};

struct stmt_postfix_t {
  char *ident;
  int32_t is_decr;
};

struct stmt_if_t {
  struct expr_t *cond;
  struct many_t *if_s;    // stmt_t
  struct many_t *else_s;  // stmt_t // nullable
};

struct stmt_while_t {
  struct expr_t *cond;
  struct many_t *s;  // stmt_t
};

extern struct stmt_t *stmt_var_decls_create(char *type, struct many_t *decls);
extern struct var_decl_t *var_decl_create(char *ident, struct expr_t *e);

extern struct stmt_t *stmt_assign_create(char *ident, struct expr_t *e);
extern struct stmt_t *stmt_postfix_create(char *ident, int is_decr);
extern struct stmt_t *stmt_return_create(struct expr_t *e);
extern struct stmt_t *stmt_block_create(struct many_t *stmts);
extern struct stmt_t *stmt_expr_create(struct expr_t *e);
extern struct stmt_t *stmt_if_create(struct expr_t *cond, struct many_t *if_s,
                                     struct many_t *else_s);
extern struct stmt_t *stmt_while_create(struct expr_t *cond, struct many_t *s);

extern void stmt_free(void *s);
extern void var_decl_free(void *d);

#endif
