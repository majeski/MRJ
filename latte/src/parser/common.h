#ifndef COMMON__H
#define COMMON__H

#include <assert.h>
#include <stdlib.h>

extern int mem_error;

#define CHECK_NULL(ptr) \
  do {                  \
    if (ptr == NULL) {  \
      mem_error = 1;    \
      return NULL;      \
    }                   \
  } while (0)

#include "def.h"
#include "expr.h"
#include "field_get.h"
#include "many.h"
#include "stmt.h"
#include "type.h"

#endif
