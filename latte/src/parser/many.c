#include "common.h"

struct many_t *many_create(void *elem) {
  struct many_t *m = malloc(sizeof(struct many_t));
  CHECK_NULL(m);
  m->next = NULL;
  m->elem = elem;
  return m;
}

void many_free(struct many_t *many, void (*free_f)(void *)) {
  assert(free_f);
  if (many == NULL) {
    return;
  }
  many_free(many->next, free_f);
  (*free_f)(many->elem);
  free(many);
}

struct many_t *many_add(void *elem, struct many_t *next) {
  struct many_t *many = many_create(elem);
  CHECK_NULL(many);
  many->next = next;
  return many;
}
