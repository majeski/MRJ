OBJECTS = latte.tab.o lex.yy.o common.o def.o expr.o field_get.o many.o type.o stmt.o
CFLAGS = -Wall -std=gnu11

libparse.a: $(OBJECTS)
	ar rcs libparse.a $(OBJECTS)

%.o: %.c %.h
	$(CC) $(CFLAGS) -c $<

lex.yy.c: latte.l latte.tab.h
	flex latte.l

latte.tab.c latte.tab.h: latte.y def.h expr.h many.h stmt.h
	bison -d latte.y

clean:
	rm -rf libparse.a $(OBJECTS) latte.tab.{c,h} lex.yy.c
