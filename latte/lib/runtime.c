#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void printInt(int x) {
  printf("%d", x);
}

void printString(char *s) {
  printf("%s", s);
}

void error() {
  exit(-1);
}

int readInt() {
  int x;
  scanf("%d", &x);
  return x;
}

char *readString() {
  char *buf = NULL;
  size_t size = 0;
  if (getline(&buf, &size, stdin) == -1) {
    return NULL;
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
