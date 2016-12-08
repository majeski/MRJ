#ifndef DEF__H
#define DEF__H

#include <stdint.h>

extern const int32_t DEF_TYPE_FUNC;

struct def_t {
  int32_t type;
  void *d;
};

struct def_func_t {
  char *ret_type;
  char *ident;
  struct many_t *args;  // func_arg_t
  struct many_t *stmts;
};

struct func_arg_t {
  char *type;
  char *ident;
};

extern void def_free(void *def);
extern void func_arg_free(void *farg);

extern struct def_t *def_func_create(char *ret_type, char *ident,
                                     struct many_t *args, struct many_t *stmts);

extern struct func_arg_t *func_arg_create(char *type, char *ident);

#endif
