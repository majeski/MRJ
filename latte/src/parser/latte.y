%{
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>

extern int yylex();
extern FILE *yyin;
extern int yyparse();
extern int line_num;

extern int parse(int input_fd);
extern void free_parsed_defs();
extern struct many_t *parsed_defs;

void yyerror(const char *s);

#define YYERROR_VERBOSE 1

#include "ast.h"
%}

%union {
  char *str;
  struct def_t *def;
  struct class_t *class;
  struct class_member_t *class_member;
  struct func_t *func;
  struct var_t *var;
  struct expr_t *expr;
  struct many_t *many;
  struct stmt_t *stmt;
  struct var_decl_t *var_decl;
  struct field_get_t *field_get;
}

%token <str> IDENT "identifier"
%token <str> BUILTIN_TYPE
%token <str> UNKNOWN "unknown token"

%token <str> LIT_INT "integer literal";
%token <str> LIT_STR "string literal";
%token <str> LIT_BOOL "boolean literal";
%token LIT_NULL "null literal"

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
%right '['
%left '.'

%token BRACKETS "[]"
%token CLASS "class definition"
%token ELSE "else"
%token EXTENDS "extends <superclass>"
%token FOR "for"
%token IF "if statement"
%token NEW "new"
%token RETURN "return statement"
%token WHILE "while statement"

%type <many> defs "list of definitions";
%type <def> def "definition";

%type <class> class_def "class definition"
%type <many> class_members "class members";
%type <class_member> class_member "class member";

%type <func> func_def "function definition";
%type <many> f_args "function arguments";

%type <var> var "variable declaration";

%type <many> body "function body";

%type <many> stmts "list of statements";
%type <stmt> stmt_block "block of statements";
%type <stmt> stmt "statement";

%type <many> var_inits;
%type <var_decl> var_init;

%type <many> exprs "list of expressions";
%type <expr> expr "expression";

%type <field_get> field_get;

%type <str> type "type";
%type <str> simple_type "simple_type";
%%
program: defs { parsed_defs = $1; }

defs: def { $$ = many_create($1); }
    | def defs { $$ = many_add($1, $2); }

def: func_def { $$ = def_func_create($1); }
   | class_def { $$ = def_class_create($1); }

class_def: CLASS IDENT '{' class_members '}' {
            $$ = class_create($2, NULL, $4);
         }
         | CLASS IDENT EXTENDS IDENT '{' class_members '}' {
            $$ = class_create($2, $4, $6);
         }

class_members: /* empty */ { $$ = NULL; }
             | class_member class_members { $$ = many_add($1, $2); }

class_member: func_def { $$ = class_member_func_create($1); }
            | var ';' { $$ = class_member_var_create($1); }

func_def: type IDENT '(' f_args ')' body { $$ = func_create($1, $2, $4, $6); }

f_args: /* empty */ { $$ = NULL; }
      | var { $$ = many_create($1); }
      | var ',' f_args { $$ = many_add($1, $3); }

var: type IDENT { $$ = var_create($1, $2); }

stmt_block: '{' '}' { $$ = stmt_block_create(NULL); }
          |'{' stmts '}' { $$ = stmt_block_create($2); }

body: '{' '}' { $$ = NULL; }
    | '{' stmts '}' { $$ = $2; }

stmts: stmt { $$ = many_create($1); }
     | stmt stmts { $$ = many_add($1, $2); }

stmt: /* empty */ ';' { $$ = stmt_empty_create(); }
    | type var_inits ';' { $$ = stmt_var_decls_create($1, $2); }
    | field_get '=' expr ';' { $$ = stmt_assign_create($1, $3); }
    | field_get INCR ';' { $$ = stmt_postfix_create($1, 0); }
    | field_get DECR ';' { $$ = stmt_postfix_create($1, 1); }
    | RETURN ';' { $$ = stmt_return_create(NULL); }
    | RETURN expr ';' { $$ = stmt_return_create($2); }
    | stmt_block { $$ = $1; }
    | expr ';' { $$ = stmt_expr_create($1); }
    | IF '(' expr ')' stmt ELSE stmt { $$ = stmt_if_create($3, $5, $7); } /* shift/reduce */
    | IF '(' expr ')' stmt { $$ = stmt_if_create($3, $5, NULL); }
    | WHILE '(' expr ')' stmt { $$ = stmt_while_create($3, $5); }
    | FOR '(' type IDENT ':' expr ')' stmt { $$ = stmt_for_create($3, $4, $6, $8); }

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
    | NEW simple_type '[' expr ']' { $$ = expr_new_create($2, $4); } /* shift/reduce */
    | NEW simple_type { $$ = expr_new_create($2, NULL); }
    | field_get '(' exprs ')' { $$ = expr_call_create($1, $3); }
    | field_get { $$ = expr_field_get_create($1); }
    | '(' IDENT ')' LIT_NULL { $$ = expr_lit_create(EXPR_TYPE_LIT_NULL, $2); } /* shift/reduce */
    | LIT_INT { $$ = expr_lit_create(EXPR_TYPE_LIT_INT, $1); }
    | LIT_STR { $$ = expr_lit_create(EXPR_TYPE_LIT_STR, $1); }
    | LIT_BOOL { $$ = expr_lit_create(EXPR_TYPE_LIT_BOOL, $1); }
    | LIT_NULL { $$ = expr_lit_create(EXPR_TYPE_LIT_NULL, NULL); }

field_get: expr '.' IDENT { $$ = field_get_create($1, $3); }
         | expr '[' expr ']' { $$ = field_get_idx_create($1, $3); }
         | IDENT { $$ = field_get_create(NULL, $1); }

type: simple_type BRACKETS { $$ = array_type_create($1); }
    | simple_type

simple_type: BUILTIN_TYPE { $$ = $1; }
           | IDENT { $$ = $1; }
%%

struct many_t *parsed_defs = NULL;

void yyerror(const char *s) {
  fprintf(stderr, "ERROR\nline %d: %s\n", line_num, s);
}

int parse(int input_fd) {
  parsed_defs = NULL;
	yyin = fdopen(input_fd, "r");
	if (yyin == NULL) {
		fprintf(stderr, "ERROR\n[parser] Couldn't open input file");
		return 1;
	}

  if (yyparse() == 0) {
    if (mem_error) {
      fprintf(stderr, "ERROR\n[parser] Memory error");
      parsed_defs = NULL;
      return 1;
    }
    return 0;
  }
  return 1;
}

void free_parsed_defs() {
  many_free(parsed_defs, def_free);
}
