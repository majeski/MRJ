#include "common.h"

const int32_t DEF_TYPE_FUNC = 1;

struct def_t *def_create(int32_t type, void *def);

struct def_t *def_func_create(char *ret_type, char *ident, struct many_t *args,
                              struct stmt_t *stmt) {
  struct def_func_t *d = malloc(sizeof(struct def_func_t));
  CHECK_NULL(d);
  d->ret_type = ret_type;
  d->ident = ident;
  d->args = args;
  d->stmt = stmt;
  return def_create(DEF_TYPE_FUNC, d);
}

struct def_t *def_create(int32_t type, void *def) {
  struct def_t *d = malloc(sizeof(struct def_t));
  CHECK_NULL(d);
  d->type = type;
  d->d = def;
  return d;
}

void def_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  int32_t type = ((struct def_t *)ptr)->type;
  void *d = ((struct def_t *)ptr)->d;
  if (d == NULL) {
    goto end;
  }

  if (type == DEF_TYPE_FUNC) {
    struct def_func_t *def = (struct def_func_t *)d;
    free(def->ret_type);
    free(def->ident);
    many_free(def->args, func_arg_free);
    stmt_free(def->stmt);
  } else {
    assert(0);
    exit(-1);
  }

end:
  free(ptr);
}

struct func_arg_t *func_arg_create(char *type, char *ident) {
  struct func_arg_t *a = malloc(sizeof(struct func_arg_t));
  CHECK_NULL(a);
  a->type = type;
  a->ident = ident;
  return a;
}

void func_arg_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct func_arg_t *a = (struct func_arg_t *)ptr;
  free(a->type);
  free(a->ident);
  free(ptr);
}
