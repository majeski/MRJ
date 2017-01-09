#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void printInt(int x) { printf("%d\n", x); }

void printString(char *s) { puts(s); }

void error() {
  puts("runtime error");
  exit(EXIT_FAILURE);
}

int readInt() {
  int x;
  scanf("%d", &x);
  getchar();
  return x;
}

char *readString() {
  char *buf = NULL;
  size_t size = 0;

  getline(&buf, &size, stdin);
  if (buf == NULL || strlen(buf) == 0) {
    if (buf == NULL) {
      buf = malloc(1);
    }
    if (buf != NULL) {
      buf[0] = 0;
    }
    return buf;
  }

  size = strlen(buf);
  if (buf[size - 1] == '\n') {
    buf[size - 1] = 0;
  }
  return buf;
}

char *concatenate(char *lhs, char *rhs) {
  size_t lsize = strlen(lhs);
  char *buf = malloc(lsize + strlen(rhs) + 1);
  if (buf == NULL) {
    return NULL;
  }
  strcpy(buf, lhs);
  strcpy(buf + lsize, rhs);
  return buf;
}
