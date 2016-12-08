%{
#include "latte.tab.h"

extern int mem_error;
int line_num = 1;

#define CE do { if (yylval.str == NULL) mem_error = 1; } while (0)
%}

%START VAR
%%
\n  line_num++;
[ \t] ; // whitespace

"<=" { BEGIN 0; return LE; }
">=" { BEGIN 0; return GE; }
"==" { BEGIN 0; return EQ; }
"!=" { BEGIN 0; return NEQ; }
"&&" { BEGIN 0; return AND; }
"||" { BEGIN 0; return OR; }
[-+*/%=,;{}()<>] { BEGIN 0; return yytext[0]; }

<VAR>"++" { BEGIN 0; return INCR; }
<VAR>"--" { BEGIN 0; return DECR; }

(true|false) { yylval.str = strdup(yytext); CE; return LIT_BOOL; }
[0-9]+ { yylval.str = strdup(yytext); CE; return LIT_INT; }
\"[^"]*\" {
  yylval.str = strndup(yytext + 1, strlen(yytext) - 2);
  CE;
  return LIT_STR;
}

if { return IF; }
else { return ELSE; }
while { return WHILE; }
return { return RETURN; }

(int|string|boolean|void) { yylval.str = strdup(yytext); CE; return TYPE; }
[a-zA-Z_][a-zA-Z_0-9]* { yylval.str = strdup(yytext); CE; BEGIN VAR; return IDENT; }
. { yylval.str = strdup(yytext); CE; return UNKNOWN; }
%%