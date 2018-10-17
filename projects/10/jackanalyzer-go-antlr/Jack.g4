grammar Jack;

clazz : 'class' className
        '{' classVarDec* subroutineDec* '}';

classVarDec : scopeType dataType varList ';';

varList : varName (',' varName)*;

scopeType : 'static' | 'field';

dataType : 'int' | 'char' | 'boolean' | className;

subroutineDec : subroutineType returnType subroutineName '(' parameterList ')' subroutineBody;

subroutineType : 'constructor' | 'function' | 'method';
returnType : 'void' | dataType;

parameterList : (typedVar (',' typedVar)*)?;

typedVar : dataType varName;

subroutineBody : '{' varDec* statements '}';

varDec : 'var' dataType varList ';';

className : ID;

subroutineName : ID;

varName : ID;

statements : statement*;

statement : letStatement
          | ifStatement
          | whileStatement
          | doStatement
          | returnStatement
          ;

letStatement : 'let' varName arrayIndexing? '=' expression ';';
arrayIndexing : '[' expression ']';

ifStatement : 'if' '(' expression ')'
              '{' statements '}'
              elseClause?
            ;

elseClause : 'else' '{' statements '}';

whileStatement : 'while' '(' expression ')'
                 '{' statements '}'
               ;

doStatement : 'do' subroutineCall ';';

returnStatement : 'return' expression? ';' ;

expression : 
expression bop=('*'|'/') expression # BinaryExpr
| expression bop=('+'|'-') expression # BinaryExpr
| expression bop=('&'| '|') expression # BinaryExpr
| expression bop=('>'|'<'|'=') expression # BinaryExpr
| term # NormalExpr
;

term : INTEGER          #intergerTerm
     | STRING           #stringTerm
     | keywordConstant  #ruleTerm
     | varName          #ruleTerm
     | arrayExpr        #ruleTerm
     | subroutineCall   #ruleTerm
     | parenthesesExpr  #ruleTerm
     | unaryExpr        #ruleTerm
     ;

parenthesesExpr : '(' expression ')';

unaryExpr : unaryOp term;

arrayExpr : varName '[' expression ']';

subroutineCall : (qualifier '.')? subroutineName '(' expressionList ')';

qualifier : className | varName;

expressionList : (expression (',' expression)* )?;

op : '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=';

//higerOP: '*' | '/' | '&'; 
//lowerOP: '+' | '-' | '|' | '>' | '<' | '=';

unaryOp : '-' | '~';

keywordConstant : 'true' | 'false' | 'null' | 'this';

STRING : '"' STRING_CHAR* '"';
fragment STRING_CHAR :   ~["\\] | ESCAPED_CHAR ;
fragment ESCAPED_CHAR : '\\' [btnfr"'\\];

INTEGER      : [0-9]+;
ID           : [a-zA-Z_][a-zA-Z0-9_]*;
WS           :  [ \t\r\n\u000C]+ -> skip;
COMMENT      :   '/*' .*? '*/' -> skip;
LINE_COMMENT : '//' ~[\r\n]* -> skip;
