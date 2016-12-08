#ifndef MANY__H
#define MANY__H

struct many_t {
  struct many_t *next;
  void *elem;
};

struct many_t *many_create(void *elem);
void many_free(struct many_t *many, void (*free_f)(void *));
struct many_t *many_add(void *elem, struct many_t *many);

#endif
