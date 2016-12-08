%{
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

extern int yylex();
extern int yyparse();
extern int line_num;

extern int parse();
extern void free_parsed_defs();
extern struct many_t *parsed_defs;

void yyerror(const char *s);

#define YYERROR_VERBOSE 1

#include "ast.h"
%}

%union {
  char *str;
  struct def_t *def;
  struct expr_t *expr;
  struct func_arg_t *f_arg;
  struct many_t *many;
  struct stmt_t *stmt;
  struct var_decl_t *var_decl;
}

%token <str> IDENT "identifier"
%token <str> TYPE "type"
%token <str> UNKNOWN "unknown token"

%token <str> LIT_INT "integer literal";
%token <str> LIT_STR "string literal";
%token <str> LIT_BOOL "boolean literal";

%token INCR "++"
%token DECR "--"

%token OR "||"
%token AND "&&"
%token LE "<="
%token GE ">="
%token EQ "=="
%token NEQ "!="

%left OR
%left AND
%nonassoc '<' '>' LE GE EQ NEQ
%left '+' '-'
%left '*' '/' '%'
%left UNOT UNEG

%token RETURN "return statement"
%token IF "if statement"
%token ELSE "else"
%token WHILE "while statement"

%type <many> defs "list of definitions";
%type <def> def "definition";

%type <many> f_args "function arguments";
%type <f_arg> f_arg "function argument";

%type <many> stmts "list of statements";
%type <many> block;
%type <stmt> stmt_block "block of statements";
%type <stmt> stmt "statement";

%type <many> var_inits;
%type <var_decl> var_init;

%type <many> exprs "list of expressions";
%type <expr> expr "expression";

%%
program: defs { parsed_defs = $1; }

defs: def { $$ = many_create($1); }
    | def defs { $$ = many_add($1, $2); }

def: TYPE IDENT '(' f_args ')' body {
      $$ = def_func_create($1, $2, $4, $6);
   }

f_args: /* empty */ { $$ = NULL; }
      | f_arg { $$ = many_create($1); }
      | f_arg ',' f_args { $$ = many_add($1, $3); }

f_arg: TYPE IDENT { $$ = func_arg_create($1, $2); }

stmt_block: '{' '}' { $$ = stmt_block_create(NULL); }
          |'{' stmts '}' { $$ = stmt_block_create($2); }

body: '{' '}' { $$ = NULL; }
    | '{' stmts '}' { $$ = $2; }

stmts: stmt { $$ = many_create($1); }
     | stmt stmts { $$ = many_add($1, $2); }

stmt: TYPE var_inits ';' { $$ = stmt_var_decls_create($1, $2); }
    | IDENT '=' expr ';' { $$ = stmt_assign_create($1, $3); }
    | IDENT INCR ';' { $$ = stmt_postfix_create($1, 0); }
    | IDENT DECR ';' { $$ = stmt_postfix_create($1, 1); }
    | RETURN ';' { $$ = stmt_return_create(NULL); }
    | RETURN expr ';' { $$ = stmt_return_create($2); }
    | stmt_block { $$ = $1; }
    | expr ';' { $$ = stmt_expr_create($1); }
    | IF '(' expr ')' body ELSE body { $$ = stmt_if_create($3, $5, $7); }
    | IF '(' expr ')' body { $$ = stmt_if_create($3, $5, NULL); }
    | WHILE '(' expr ')' body { $$ = stmt_while_create($3, $5); }

var_inits: var_init { $$ = many_create($1); }
         | var_init ',' var_inits { $$ = many_add($1, $3); }

var_init: IDENT '=' expr { $$ = var_decl_create($1, $3); }
        | IDENT { $$ = var_decl_create($1, NULL); }

exprs: /* empty */ { $$ = NULL; }
     | expr { $$ = many_create($1); }
     | expr ',' exprs { $$ = many_add($1, $3); }

expr: expr OR expr { $$ = expr_binop_create($1, $3, "||"); }
    | expr AND expr { $$ = expr_binop_create($1, $3, "&&"); }
    | expr '<' expr { $$ = expr_binop_create($1, $3, "<"); }
    | expr '>' expr { $$ = expr_binop_create($1, $3, ">"); }
    | expr LE expr { $$ = expr_binop_create($1, $3, "<="); }
    | expr GE expr { $$ = expr_binop_create($1, $3, ">="); }
    | expr EQ expr { $$ = expr_binop_create($1, $3, "=="); }
    | expr NEQ expr { $$ = expr_binop_create($1, $3, "!="); }
    | expr '+' expr { $$ = expr_binop_create($1, $3, "+"); }
    | expr '-' expr { $$ = expr_binop_create($1, $3, "-"); }
    | expr '*' expr { $$ = expr_binop_create($1, $3, "*"); }
    | expr '/' expr { $$ = expr_binop_create($1, $3, "/"); }
    | expr '%' expr { $$ = expr_binop_create($1, $3, "%"); }
    | '-' expr %prec UNEG { $$ = expr_unary_create('-', $2); }
    | '!' expr %prec UNOT { $$ = expr_unary_create('!', $2); }
    | '(' expr ')' { $$ = $2; }
    | IDENT '(' exprs ')' { $$ = expr_call_create($1, $3); }
    | IDENT { $$ = expr_ident_create($1); }
    | LIT_INT { $$ = expr_lit_create(EXPR_TYPE_LIT_INT, $1); }
    | LIT_STR { $$ = expr_lit_create(EXPR_TYPE_LIT_STR, $1); }
    | LIT_BOOL { $$ = expr_lit_create(EXPR_TYPE_LIT_BOOL, $1); }
%%

struct many_t *parsed_defs = NULL;

void yyerror(const char *s) {
  printf("line %d: %s\n", line_num, s);
}

int parse() {
  parsed_defs = NULL;
  if (yyparse() == 0) {
    if (mem_error) {
      printf("Memory error");
      parsed_defs = NULL;
      return 1;
    }
  }
  return 0;
}

void free_parsed_defs() {
  many_free(parsed_defs, def_free);
}
