#include "common.h"

const int DEF_TYPE_FUNC = 1;
const int DEF_TYPE_CLASS = 2;

const int CLASS_MEMBER_TYPE_FUNC = 10;
const int CLASS_MEMBER_TYPE_VAR = 11;

struct def_t *def_create(int32_t type, void *def);

struct def_t *def_func_create(struct func_t *f) {
  return def_create(DEF_TYPE_FUNC, f);
}

struct def_t *def_class_create(struct class_t *c) {
  return def_create(DEF_TYPE_CLASS, c);
}

struct def_t *def_create(int32_t type, void *def) {
  struct def_t *d = malloc(sizeof(struct def_t));
  CHECK_NULL(d);
  d->type = type;
  d->d = def;
  return d;
}

struct func_t *func_create(char *ret_type, char *ident, struct many_t *args,
                           struct many_t *stmts) {
  struct func_t *f = malloc(sizeof(struct func_t));
  CHECK_NULL(f);
  f->ret_type = ret_type;
  f->ident = ident;
  f->args = args;
  f->stmts = stmts;
  return f;
}

struct var_t *var_create(char *type, char *ident) {
  struct var_t *v = malloc(sizeof(struct var_t));
  CHECK_NULL(v);
  v->type = type;
  v->ident = ident;
  return v;
}

struct class_t *class_create(char *name, char *super, struct many_t *members) {
  struct class_t *c = malloc(sizeof(struct class_t));
  CHECK_NULL(c);
  c->name = name;
  c->superclass = super;
  c->members = members;
  return c;
}

struct class_member_t *class_member_create(int32_t type, void *m);

struct class_member_t *class_member_func_create(struct func_t *f) {
  return class_member_create(CLASS_MEMBER_TYPE_FUNC, f);
}

struct class_member_t *class_member_var_create(struct var_t *v) {
  return class_member_create(CLASS_MEMBER_TYPE_VAR, v);
}

struct class_member_t *class_member_create(int32_t type, void *m) {
  struct class_member_t *cm = malloc(sizeof(struct class_member_t));
  CHECK_NULL(cm);
  cm->type = type;
  cm->m = m;
  return cm;
}

void func_free(void *ptr);
void class_free(void *ptr);
void class_member_free(void *ptr);
void var_free(void *ptr);

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
    func_free(d);
  } else if (type == DEF_TYPE_CLASS) {
    class_free(d);
  } else {
    assert(0);
    exit(-1);
  }

end:
  free(ptr);
}

void func_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct func_t *f = ptr;
  free(f->ret_type);
  free(f->ident);
  many_free(f->args, var_free);
  many_free(f->stmts, stmt_free);
  free(f);
}

void class_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct class_t *c = ptr;
  free(c->name);
  many_free(c->members, class_member_free);
  free(c);
}

void class_member_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  int32_t type = ((struct class_member_t *)ptr)->type;
  void *m = ((struct class_member_t *)ptr)->m;
  if (m == NULL) {
    goto end;
  }

  if (type == CLASS_MEMBER_TYPE_FUNC) {
    func_free(m);
  } else if (type == CLASS_MEMBER_TYPE_VAR) {
    var_free(m);
  } else {
    assert(0);
    exit(-1);
  }

end:
  free(ptr);
}

void var_free(void *ptr) {
  if (ptr == NULL) {
    return;
  }

  struct var_t *v = ptr;
  free(v->type);
  free(v->ident);
  free(v);
}
