#ifndef FIELD_GET__H
#define FIELD_GET__H

struct field_get_t {
  char *ident;
  struct field_get_t *field;
};

extern void field_get_free(void *fg);

extern struct field_get_t *field_get_create(char *ident,
                                            struct field_get_t *field);

#endif
