#ifndef DEF__H
#define DEF__H

#include <stdint.h>

extern const int DEF_TYPE_FUNC;
extern const int DEF_TYPE_CLASS;

struct def_t {
  int32_t type;
  void *d;  // func_t | class_t
};

extern const int CLASS_MEMBER_TYPE_FUNC;
extern const int CLASS_MEMBER_TYPE_VAR;

struct class_t {
  char *name;
  char *superclass;
  struct many_t *members;  // class_member_t
};

struct class_member_t {
  int32_t type;
  void *m;  // func_t | var_t
};

struct func_t {
  char *ret_type;
  char *ident;
  struct many_t *args;   // var_t
  struct many_t *stmts;  // stmt_t
};

struct var_t {
  char *type;
  char *ident;
};

extern void def_free(void *def);

extern struct def_t *def_func_create(struct func_t *f);
extern struct def_t *def_class_create(struct class_t *c);

extern struct func_t *func_create(char *ret_type, char *ident,
                                  struct many_t *args, struct many_t *stmts);

extern struct class_t *class_create(char *name, char *super,
                                    struct many_t *members);
extern struct class_member_t *class_member_func_create(struct func_t *f);
extern struct class_member_t *class_member_var_create(struct var_t *v);

extern struct var_t *var_create(char *type, char *ident);

#endif
