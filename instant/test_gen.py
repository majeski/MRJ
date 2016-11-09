#!/usr/bin/python2.7

import random
import datetime

def generate_program():
    vars = set()
    n = random.randint(5, 10)
    res = []
    for i in xrange(0, 20):
        if res:
            res += [";", "\n"]
        res += generate_stmt(vars)
    return res

def generate_stmt(vars):
    x = random.randint(1, 3)
    if x == 1:
        ident = generate_ident(vars)
        res = [ident] + ["="] + generate_expr(vars) + [";", ident]
        vars.add(ident)
        return res
    return generate_expr(vars)

def generate_expr(vars, level = 1):
    x = random.randint(1, 3)
    if x == 1 or level > 10:
        return [generate_val(vars)]

    op = random.choice(["+", "-", "*"])
    res = generate_expr(vars, level + 1) + [op] + generate_expr(vars, level + 1)
    if random.randint(1, 4) == 1:
        return ["("] + res + [")"]
    return res

def generate_val(vars):
    beg = 1
    if not vars:
        beg = 2
    x = random.randint(beg, 2)
    if x == 1:
        return "%s" % list(vars)[random.randint(0, len(vars) - 1)]
    elif x == 2:
        return "%d" % random.randint(0, 20)

def generate_ident(vars):
    ident = "asdASD123%d" % len(vars)
    return ident

if __name__ == '__main__':
    random.seed(datetime.datetime.now())
    program = generate_program()
    print ' '.join(program)
