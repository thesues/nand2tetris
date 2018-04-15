use std::io::Write;
use std::env;
use std::vec::Vec;
use std::process;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};
use std::fs::OpenOptions;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum TokenType{
    KEYWORD(String),
    SYMBOL(String),
    INTEGER(u16),
    STRING(String),
    IDENTIFIER(String),
}

impl PartialEq for TokenType {
    fn eq(&self, other: &TokenType) -> bool {
        match (self, other) {
            (&TokenType::KEYWORD(ref a), &TokenType::KEYWORD(ref b)) => {
                a == b
            },
            (&TokenType::SYMBOL(ref a), &TokenType::SYMBOL(ref b)) => {
                a == b
            },
            (&TokenType::IDENTIFIER(ref _a), &TokenType::IDENTIFIER(ref _b)) => {
                true
            },
            (&TokenType::INTEGER(ref _a), &TokenType::INTEGER(ref _b)) => {
                true
            },
            (&TokenType::STRING(ref _a), &TokenType::STRING(ref _b)) => {
                true
            },
            _ => false
        }
    }
}

fn escape_word(s :&str) -> String {
    match s {
        "&" => String::from("&amp;"),
        ">" => String::from("&gt;"),
        "<" => String::from("&lt;"),
        "\"" => String::from("&quot;"),
        _ => s.to_string()
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = match self{
            &TokenType::IDENTIFIER(ref s) => format!("<identifier> {} </identifier>", escape_word(s)),
            &TokenType::KEYWORD(ref s) => format!("<keyword> {} </keyword>", escape_word(s)),
            &TokenType::INTEGER(ref i) => format!("<integerConstant> {} </integerConstant>", i),
            &TokenType::STRING(ref s) => format!("<stringConstant> {} </stringConstant>", escape_word(s)),
            &TokenType::SYMBOL(ref s) => format!("<symbol> {} </symbol>", escape_word(s))
        };
        write!(f, "{}", display)
    }
}

//class name, subroutine name
#[derive(Clone)]
enum VarSegment{
    STATIC,
    FIELD,
    ARG,
    VAR,
}

impl fmt::Display for VarSegment{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = match self{
            &VarSegment::STATIC => "STATIC",
            &VarSegment::FIELD  => "FIELD",
            &VarSegment::ARG    => "ARG",
            &VarSegment::VAR    => "VAR"
        };
        write!(f, "{}", display)
    }
}

#[derive(Clone)]
struct SymbolEntry {
    var_type :String,
    kind: VarSegment,
    number: u32,
}

impl fmt::Display for SymbolEntry{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{},{}", self.var_type, self.kind, self.number)
    }
}

impl Default for SymbolEntry{
    fn default() -> SymbolEntry{
        SymbolEntry{var_type:String::from(""), kind: VarSegment::STATIC, number:0}
    }
}
/*
impl fmt::Display for {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    }
}
*/
struct SymbolTable {
    class_var_map: HashMap<String, SymbolEntry>,
    local_var_map: HashMap<String, SymbolEntry>,
    next_field_num: u32,
    next_static_num: u32,
    next_args_num:u32,
    next_localvar_num: u32,
}

/*
static variable is shared by all the instance in the same class
vmtranslate will translate 'push static 3'
to 'push filename.static.3' */
impl SymbolTable {

    pub fn new() -> SymbolTable {
        SymbolTable{
            class_var_map: HashMap::new(),
            local_var_map: HashMap::new(),
            next_field_num:0,
            next_static_num:0,
            next_args_num:0,
            next_localvar_num:0
        }
    }

    pub fn start_subroutine(&mut self, is_method :bool) {
        self.next_args_num = if is_method {1} else {0};
        self.next_localvar_num = 0;
    }

    pub fn define(&mut self, name :&str, var_type: &str, kind: &VarSegment) {
        match kind {
            &VarSegment::FIELD => {
                self.class_var_map.insert(name.to_string(), SymbolEntry{var_type:var_type.to_string(), 
                            kind:kind.clone(), number:self.next_field_num});
                self.next_field_num += 1;
            },
            &VarSegment::STATIC => {
                self.class_var_map.insert(name.to_string(), SymbolEntry{var_type:var_type.to_string(),
                            kind:kind.clone(), number:self.next_static_num});
                self.next_static_num += 1;
            },
            &VarSegment::ARG => {
                self.local_var_map.insert(name.to_string(), SymbolEntry{var_type:var_type.to_string(),
                 kind:kind.clone(), number:self.next_args_num});
                 self.next_args_num += 1;
            },
            &VarSegment::VAR => {
                self.local_var_map.insert(name.to_string(), SymbolEntry{var_type:var_type.to_string(),
                 kind:kind.clone(), number:self.next_localvar_num});
                 self.next_localvar_num += 1;
            }
        }
    }

    pub fn var_count(&self, kind: VarSegment) -> u32 {
        match kind {
            VarSegment::FIELD => self.next_field_num,
            VarSegment::STATIC => self.next_static_num,
            VarSegment::ARG => self.next_args_num,
            VarSegment::VAR => self.next_localvar_num
        }
    }

    //== kind_of, type_of, index_of
    pub fn kind_of(&self, name: &str) -> Option<&SymbolEntry> {
        if let Some(s) = self.local_var_map.get(name) {
            //find var in local-level hashmap
            Some(s)
        } else {
            //try to find in class-level hashmap
            if let Some(s) = self.class_var_map.get(name) {
                Some(s)
            } else {
                None
            }
        }
    }
}

struct Token {
    token_type: TokenType,
}

fn base_name(filename: &str) -> &str {
    match filename.rfind(".") {
        Some(pos) => &filename[0..pos],
        None =>  "tempfile"
    }
}
//for each file
struct JackTokenizer {
    file: File,
    filename: String,
    vector: Vec<Token>
}

impl  JackTokenizer {

    //helper functions
    fn is_defined_symbol(c :char) -> bool {
        match c {
            '{'| '}' | '(' | ')' | '[' |  ']' | '.' | ',' | ';' | '+' | '-' | '*' | '/' | '&' | '|' | '<' | '>' | '=' | '~' => true,
            _ => false
        }
    }

    fn is_keyword(s :&str) -> bool {
        match s {
            "class"|"constructor"|"function"|"method"|
            "field"|"static"|"var"|"int"|"char"|"boolean"|
            "void"|"true"|"false"|"null"| "this"|"let"|"do"|
            "if"|"else"|"while"|"return" => true,
            _ => {false}
        }
    }


    fn add_to_list(vec: &mut Vec<Token>, filename: &str, t: TokenType) {
        //print to xml


        vec.push( 
            Token{
                token_type: t
            });
    }


    pub fn new(filename: &str) -> JackTokenizer{
        let file = File::open(filename).unwrap();
        return JackTokenizer{file:file, filename: filename.to_string(), vector:vec![]}
    }

    pub fn process(&mut self) {
        let mut in_comment = false;
        let mut vec :Vec<Token> = vec![];
        for line in BufReader::new(&mut self.file)
            .lines()
            .filter_map(|x| x.ok())
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            {

            //skip multiline comment
            if line.starts_with("/*") || line.starts_with("/**") {
                in_comment = true;
            }

            if line.ends_with("*/") {
                in_comment = false;
                continue;
            }

            if in_comment {
                continue;
            }
            //skip line comment
            let comment_offset = line.find("//").unwrap_or(line.len());
            let (first, _last) = line.split_at(comment_offset);

            if first.is_empty() {
                continue;
            }

            //fuck this, I can not use regex to resolve the token

            //word could be ident or keyword
            let mut word_pos: usize = 0;

            let mut number_pos: usize = 0;

            let mut string_pos: usize = 0;

            enum parse_stat {
                STRING,
                SYMBOL,
                NUMBER,
                SPACE,
                WORD,
                NONE
            }

            let mut previous_stat =  parse_stat::NONE;

            for (pos, c) in first.chars().enumerate() {
                //SYMBOL
                if JackTokenizer::is_defined_symbol(c) {
                    match previous_stat {
                        parse_stat::WORD => {
                            let word =  &first[word_pos..pos as usize];
                            if JackTokenizer::is_keyword(word) {
                                JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::KEYWORD(word.to_string()));
                            } else {
                                JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::IDENTIFIER(word.to_string()));
                            }
                        },
                        parse_stat::NUMBER => {
                            let number :u16 = first[number_pos..pos].parse().unwrap();
                            JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::INTEGER(number));
                        },
                        parse_stat::STRING => {
                            continue;
                        }
                        _ => {}
                    }
                    JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::SYMBOL(c.to_string()));
                    previous_stat = parse_stat::SYMBOL;
                }

                //STRING
                if c == '"' {
                    match previous_stat {
                        parse_stat::STRING => {
                            let s = first[(string_pos+1) as usize ..pos as usize].to_string();
                            JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::STRING(s));
                            previous_stat = parse_stat::NONE;
                        },
                        _ => {
                            string_pos = pos;
                            previous_stat = parse_stat::STRING;
                            continue;
                        }

                    }
                }

                //keyword do not have number,
                //identy can not start with a number
                //so this must be number
                if c.is_numeric() {
                    match previous_stat {
                        parse_stat::STRING => {
                            continue;
                        },
                        parse_stat::WORD => {
                            continue;
                        },
                        parse_stat::NUMBER => {
                            continue;
                        }
                        _ => {
                            previous_stat = parse_stat::NUMBER;
                            number_pos = pos;
                        }
                    }
                }

                /* a-z, A-Z, _*/
                if c.is_alphabetic() || c == '_' {
                    match previous_stat {
                        parse_stat::STRING => {
                            continue;
                        },
                        parse_stat::NONE | parse_stat::SPACE | parse_stat::SYMBOL => {
                            word_pos = pos;
                            previous_stat = parse_stat::WORD;
                        },
                        parse_stat::NUMBER => {
                            panic!("tokeniz wrong {}!", line);
                        },
                        parse_stat::WORD => {
                            continue;
                        }
                    }
                }

                if c.is_whitespace() {
                    match previous_stat {
                        parse_stat::WORD => {
                            let word =  &first[word_pos..pos as usize];
                            if JackTokenizer::is_keyword(word) {
                                JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::KEYWORD(word.to_string()));
                            } else {
                                JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::IDENTIFIER(word.to_string()));
                            }
                            previous_stat = parse_stat::SPACE;
                        },
                        parse_stat::NONE | parse_stat::SPACE | parse_stat::SYMBOL => {
                            previous_stat = parse_stat::SPACE;
                            continue;
                        },
                        parse_stat::NUMBER=>{
                            let number :u16 = first[number_pos..pos].parse().unwrap();
                            JackTokenizer::add_to_list(&mut vec, &self.filename, TokenType::INTEGER(number));
                            previous_stat = parse_stat::SPACE;
                        },
                        parse_stat::STRING=>{
                            //space is in the constant string
                            continue;
                        },
                    }
                }

            }
        }
        self.vector = vec;
    }

    pub fn output(&self) {
        let target_file_name = format!("{}T.{}", base_name(&self.filename), "xml");

        let mut target_file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(target_file_name).unwrap();

        writeln!(target_file, "<tokens>");
        for t in &self.vector {
                writeln!(target_file, "{}", t.token_type);
        }
        writeln!(target_file, "</tokens>");
    }

    pub fn has_more_tokens(&self) -> bool{
        self.vector.len() > 0
    }

    pub fn advance(&mut self) -> Option<Token> {
        if self.vector.len() > 0 {
            Some(self.vector.remove(0))
        } else {
            None
        }
    }

    pub fn back_to_vector(&mut self, t: Token) {
        self.vector.insert(0, t);
    }

    pub fn peek(&self, i :usize) -> Option<&Token> {
        if i < self.vector.len() {
            return Some(&self.vector[i])
        } else {
            None
        }
    }
}

//vec![TokenType::SYMBOL("{".to_string())
macro_rules! jack{
   ( $($t: ident : $e: expr),* ) => {{
            let mut temp_vec = Vec::new();
            $(
                match $e {
                    _ => temp_vec.push(TokenType::$t($e.to_string()))
                }
            )*
            temp_vec
    }}
}

struct JackAnalyzer {
    target_file_name :String,
    target_file: File,
    tokenizer: JackTokenizer,
    symbol_table :SymbolTable,
    //this two variable is used to indicate the currrent subroutine decleartion, NOT for funtion calls;
    current_function_name: String,
    current_function_type: String,
    class_name: String,
    next_while_label_num: u32,
    next_if_label_num: u32
}

impl JackAnalyzer {
    pub fn new(jackfilename: &str, tokenizer :JackTokenizer ) -> JackAnalyzer {
        let target_file_name = format!("{}.{}", base_name(jackfilename), "xml");
        let target_file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&target_file_name).unwrap();
        let st = SymbolTable::new();
        return JackAnalyzer{
            target_file_name: target_file_name, 
            target_file: target_file, 
            tokenizer:tokenizer, symbol_table:st,
            current_function_name: String::default(),
            current_function_type: String::default(),
            class_name: String::default(),
            next_while_label_num : 0,
            next_if_label_num: 0
        };
    }


    fn eat(&mut self, token_type_vec: Vec<TokenType>) -> Option<TokenType> {
        let source_token = self.tokenizer.advance().unwrap();
        for i in &token_type_vec {
            if source_token.token_type == *i {
                return Some(source_token.token_type);
            }
        }
        //if the pop failed, I will push the token back
        /*
        println!("I GOT a {},", source_token.token_type);
        for i in &token_type_vec {
            println!("expected {}", i);
        }
        */
        self.tokenizer.back_to_vector(source_token);
        return None;
    }

    fn eat_force(&mut self) -> Option<TokenType> {
        match self.tokenizer.advance() {
            Some(t) => Some(t.token_type),
            None => None
        }
    }


    fn peek(&self, i: usize) -> Option<&TokenType> {
        match self.tokenizer.peek(i) {
            Some(t) => {
                Some(&t.token_type)
            },
            None => None
        }
    }


    fn compile_class(&mut self) {

        //need macro to simplifiy the code
        self.eat(jack!(KEYWORD:"class")).unwrap();

        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        match valid_token {
            TokenType::IDENTIFIER(s) => {
                self.class_name = s;
            },
            _ => panic!()
        }

        //let valid_token = self.eat(vec![TokenType::SYMBOL("{".to_string())]).unwrap();
        self.eat(jack!(SYMBOL:"{")).unwrap();

        //0 or * class var declares
        while self.compile_class_var_dec(){};


        //0 or * class subroutine

        while self.compile_subroutine(){};

        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();
    }

    //  classVarDec*
    //  if return false, means no parser,
    // if return true, means could have more to parse
    fn compile_class_var_dec(&mut self) -> bool{


        let mut symbol_name :String;
        let symbol_kind :VarSegment;
        let symbol_type :String;
        //static | field
        let valid_token = match self.eat(jack!(KEYWORD:"static", KEYWORD:"field")) {
            Some(valid_token) => valid_token,
            None => {return false}
        };
        symbol_kind = match valid_token {
            TokenType::KEYWORD(ref s) if s == "static" => VarSegment::STATIC,
            TokenType::KEYWORD(ref s) if s == "field" => VarSegment::FIELD,
            _ => {panic!();}
        };

        //type
        let valid_token = self.eat(jack!(KEYWORD:"int", KEYWORD:"char", KEYWORD:"boolean", IDENTIFIER:"")).unwrap();

        symbol_type = match valid_token {
            TokenType::KEYWORD(s) => s,
            TokenType::IDENTIFIER(s) => s,
            _ => {panic!();}
        };

        //varname
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        symbol_name = match valid_token  {
            TokenType::IDENTIFIER(s) => s,
            _ => {panic!();}
        };

        self.symbol_table.define(&symbol_name, &symbol_type, &symbol_kind);


        //writeln!(self.target_file, "<codewriter>{},{}</codewriter>", symbol_name, self.symbol_table.kind_of(&symbol_name).unwrap());

        //(, varname)*
        loop {
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(valid_token) => valid_token,
                None => {break;}
            }; 

            let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
            symbol_name = match valid_token {
                TokenType::IDENTIFIER(s) => s,
                _ => {panic!();}
            };
            self.symbol_table.define(&symbol_name, &symbol_type, &symbol_kind);

        }

        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();
        true
    }

    fn compile_subroutine(&mut self) -> bool{
        //constructor, function, method
        let valid_token = match self.eat(jack!(KEYWORD:"constructor", KEYWORD:"function", KEYWORD:"method")) {
            Some(valid_token) => valid_token,
            None => {return false}
        };

        let mut is_method :bool = false;

        self.current_function_type = match valid_token {
            TokenType::KEYWORD(s) => {
                if s == "method" {
                    is_method = true;
                }
                s
            },
            _ => panic!()
        };


        //start a new local symbol table, reset 
        self.symbol_table.start_subroutine(is_method);
        self.next_while_label_num = 0;
        self.next_if_label_num = 0;

        //void , type => int, boolean, char, ident
        self.eat(jack!(KEYWORD:"void", KEYWORD:"int", KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")).unwrap();

        //subroutine name
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        self.current_function_name = match valid_token {
            TokenType::IDENTIFIER(s) => {
                format!("{}.{}", self.class_name, s)
            } 
            _ => panic!()
        };

        //symbol '('
        let valid_token = self.eat(jack!(SYMBOL:"(")).unwrap();

        //one parameter list or none
        self.compile_parameter_list();

        //symbol ')'
        let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();

        self.compile_subroutine_body();

        true
    }

    fn compile_subroutine_body(&mut self) {
        //symbol '{'
         self.eat(jack!(SYMBOL:"{")).unwrap();


        while self.compile_var_dec() {}

        let var_count = self.symbol_table.var_count(VarSegment::VAR);
        self.write_function(var_count);

        if self.current_function_type == "constructor" {
            let field_count = self.symbol_table.var_count(VarSegment::FIELD);
            //allocate memory in heap for all field variables
            self.write_push_constant(field_count);
            self.write_call("Memory.alloc", 1);
            //assgine THIS to heap
            self.write_pop_this();
        }

        self.compile_statements();

        //symbol '}'
        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();

    }
    fn compile_parameter_list(&mut self) {
        let mut symbol_name :String;
        let mut symbol_type :String;
        //type
        let valid_token = match self.eat(jack!(KEYWORD:"void", KEYWORD:"int", 
                                         KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")) { 
            Some(valid_token) => valid_token,
            None => {
                return;
            }
        };

        symbol_type = match valid_token {
            TokenType::KEYWORD(s) => s,
            TokenType::IDENTIFIER(s) => s,
            _ => panic!()
        };

        //varname
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();

        symbol_name = match valid_token {
            TokenType::IDENTIFIER(s) => s,
            _ => {panic!()}
        };

        self.symbol_table.define(&symbol_name, &symbol_type, &VarSegment::ARG);

        //(, type varname)
        loop {
            //,
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(valid_token) => valid_token,
                None => {break;}
            };

            //type
            let valid_token = self.eat(jack!(KEYWORD:"void", KEYWORD:"int",
                                            KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")).unwrap();


            symbol_type = match valid_token {
                TokenType::KEYWORD(s) => s,
                TokenType::IDENTIFIER(s) => s,
                _ => {panic!();}
            };

            let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();

            symbol_name = match valid_token {
                TokenType::IDENTIFIER(s) => s,
                _ => {panic!();}
            };
            self.symbol_table.define(&symbol_name, &symbol_type, &VarSegment::ARG);
        }
    }

    fn compile_var_dec(&mut self) -> bool{
        let mut symbol_name :String;
        let symbol_type :String;
        //var
        let valid_token = match self.eat(jack!(KEYWORD:"var")) {
            Some(valid_token) => valid_token,
            None => {return false;}
        };

        //type
        let valid_token = self.eat(jack!(KEYWORD:"void", KEYWORD:"int",
                                        KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")).unwrap();
        symbol_type = match valid_token {
            TokenType::KEYWORD(s) => s,
            TokenType::IDENTIFIER(s) => s,
            _ => {panic!();}
        };


        //varName
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        symbol_name = match valid_token {
            TokenType::IDENTIFIER(s) => s,
            _ => {panic!();}
        };

        self.symbol_table.define(&symbol_name, &symbol_type, &VarSegment::VAR);
        loop {
            //,
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(valid_token) => valid_token,
                None => {break;}
            };
            //varname
            let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
            symbol_name = match valid_token {
                TokenType::IDENTIFIER(s) => s,
                _ => {panic!();}
            };
            self.symbol_table.define(&symbol_name, &symbol_type, &VarSegment::VAR);
        }

        //;
        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();

        true
    }


    //code writer
    fn write_push(&mut self, entry :&SymbolEntry){
        let seg = match entry.kind {
            VarSegment::ARG => "argument",
            VarSegment::FIELD => "this",
            VarSegment::STATIC => "static",
            VarSegment::VAR => "local",
        };
        writeln!(self.target_file, "push {} {}", seg, entry.number);
    }


    fn write_push_constant(&mut self, n :u32) {
        writeln!(self.target_file, "push constant {}", n);
    }

    fn write_push_this(&mut self) {
        writeln!(self.target_file, "push pointer 0");
    }

    fn write_pop_this(&mut self) {
        writeln!(self.target_file, "pop pointer 0");
    }

    fn write_pop(&mut self, entry :&SymbolEntry) {
        let seg = match entry.kind {
            VarSegment::ARG => "argument",
            VarSegment::FIELD => "this",
            VarSegment::STATIC => "static",
            VarSegment::VAR => "local"
        };
        //writeln!(self.target_file_name, "pop {} {}", seg, entry.number);
        writeln!(self.target_file, "pop {} {}", seg, entry.number);
    }

    //op should be "ADD, SUB, NEG, EQ, GT, LT, AND, OR, NOT, *
    fn write_bin_arthimetic(&mut self, op :&str) {
        match op {
            "+" => writeln!(self.target_file, "add"),
            "-" => writeln!(self.target_file, "sub"),
            "*" => writeln!(self.target_file, "call Math.multiply 2"),
            "/" => writeln!(self.target_file, "call Math.divide 2"),
            "&" => writeln!(self.target_file, "and"),
            "|" => writeln!(self.target_file, "or"),
            ">" => writeln!(self.target_file, "gt"),
            "<" => writeln!(self.target_file, "lt"),
            "=" => writeln!(self.target_file, "eq"),
            _ => panic!()
        };

    }

    fn write_uni_arthimetic(&mut self, op :&str) {
        match op {
            "~" => writeln!(self.target_file, "not"),
            "-" => writeln!(self.target_file, "neg"),
            _ => panic!()
        };
    }


    fn write_label(&mut self, label: &str) {
        writeln!(self.target_file, "label {}", label);

    }

    fn write_goto(&mut self, label: &str) {
        writeln!(self.target_file, "goto {}", label);

    }

    fn write_if(&mut self, label: &str) {
        writeln!(self.target_file, "if-goto {}", label);
    }


    fn write_call(&mut self, name: &str, nArgs: u32) {
        writeln!(self.target_file, "call {} {}", name, nArgs);
    }

    fn write_function(&mut self, nVars: u32) {
        match self.current_function_type.as_ref() {
            "method"      => {
                writeln!(self.target_file, "function {} {}", self.current_function_name, nVars);
                writeln!(self.target_file, "push argument 0");
                writeln!(self.target_file, "pop pointer 0");

            },
            _ => {
                writeln!(self.target_file, "function {} {}", self.current_function_name, nVars);

            },
        }
    }

    fn write_return(&mut self) {
        writeln!(self.target_file, "return");
    }

    fn write_ignore_return_value(&mut self) {
        writeln!(self.target_file, "pop temp 0");
    }


    fn write_pop_array_value(&mut self) {
        writeln!(self.target_file, "pop temp 0");
        writeln!(self.target_file, "pop pointer 1");
        writeln!(self.target_file, "push temp 0");
        writeln!(self.target_file, "pop that 0");
    }

    fn write_push_array_value(&mut self) {
        writeln!(self.target_file, "pop pointer 1");
        writeln!(self.target_file, "push that 0");
    }

    fn compile_statements(&mut self ) {

        loop {
            let x :u8;
            {
                x = match self.peek(0){
                    Some(&TokenType::KEYWORD(ref s)) if s == "if"    => 1,
                    Some(&TokenType::KEYWORD(ref s)) if s == "while" => 2,
                    Some(&TokenType::KEYWORD(ref s)) if s == "do"    => 3,
                    Some(&TokenType::KEYWORD(ref s)) if s == "let"   => 4,
                    Some(&TokenType::KEYWORD(ref s)) if s == "return" => 5,
                    _ => 6
                }
            }//dispose immuable self

            match x {
                1 => self.compile_if_statement(),
                2 => self.compile_while_statement(),
                3 => self.compile_do_statement(),
                4 => self.compile_let_statement(),
                5 => self.compile_return_statement(),
                _ => {
                    break;
                }
            }
        }

    }

    fn compile_if_statement(&mut self) {

        self.eat(jack!(KEYWORD:"if")).unwrap();

        self.eat(jack!(SYMBOL:"(")).unwrap();

        self.compile_expression();

        //not
        //self.write_uni_arthimetic("~");
        //based on the course, we have to use not

        self.eat(jack!(SYMBOL:")")).unwrap();

        let label_false = format!("IF_FALSE{}", self.next_if_label_num);
        let label_end = format!("IF_END{}", self.next_if_label_num);
        let label_true = format!("IF_TRUE{}", self.next_if_label_num);
        self.next_if_label_num += 1;

        self.write_if(&label_true);
        self.write_goto(&label_false);

        self.eat(jack!(SYMBOL:"{")).unwrap();

        self.write_label(&label_true);
        self.compile_statements();

        self.write_goto(&label_end);

        self.eat(jack!(SYMBOL:"}")).unwrap();

        //does it have else?
        match self.eat(jack!(KEYWORD:"else")) {
            None => {
                self.write_label(&label_false);
                self.write_label(&label_end);
                return
            },
            Some(valid_token) => {
            }
        }

        self.eat(jack!(SYMBOL:"{")).unwrap();

        self.write_label(&label_false);
        self.compile_statements();

        self.eat(jack!(SYMBOL:"}")).unwrap();
        self.write_label(&label_end);
    }

    fn compile_while_statement(&mut self) {

        self.eat(jack!(KEYWORD:"while")).unwrap();

        self.eat(jack!(SYMBOL:"(")).unwrap();

        let while_start = format!("WHILE_EXP{}", self.next_while_label_num);
        let while_end   = format!("WHILE_END{}", self.next_while_label_num);
        self.next_while_label_num += 1;

        self.write_label(&while_start);

        self.compile_expression();

        //not
        self.write_uni_arthimetic("~");
        self.write_if(&while_end);

        self.eat(jack!(SYMBOL:")")).unwrap();

        self.eat(jack!(SYMBOL:"{")).unwrap();

        self.compile_statements();

        self.eat(jack!(SYMBOL:"}")).unwrap();

        self.write_goto(&while_start);
        self.write_label(&while_end);

    }

    fn compile_do_statement(&mut self) {

        //do
        self.eat(jack!(KEYWORD:"do")).unwrap();

        self.compile_subroutine_call();
        //DO statement does not use the return value
        self.write_ignore_return_value();

        //;
        self.eat(jack!(SYMBOL:";")).unwrap();

    }

    fn compile_let_statement(&mut self) {
        //let
        self.eat(jack!(KEYWORD:"let")).unwrap();

        //ident
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();

        let symbol_entry = match valid_token {
            TokenType::IDENTIFIER(s) => {
                self.symbol_table.kind_of(&s).unwrap().clone()
            } 
            _ => panic!()
        };

        let mut is_array = false;

        match self.eat(jack!(SYMBOL:"[")) {
            Some(valid_token) => {
                //[
                is_array = true;
                self.write_push(&symbol_entry);
                self.compile_expression();
                self.write_bin_arthimetic("+");

                //]
                self.eat(jack!(SYMBOL:"]")).unwrap();
            },
            None=>{}
        }

        //=
        self.eat(jack!(SYMBOL:"=")).unwrap();

        self.compile_expression();

        if is_array {
            //pop temp 0
            //pop pointer 1
            //push temp 0
            //pop that 0
            self.write_pop_array_value();
        } else {
            self.write_pop(&symbol_entry);
        }

        //;
        self.eat(jack!(SYMBOL:";")).unwrap();

    }

    fn compile_return_statement(&mut self) {
        //return

        self.eat(jack!(KEYWORD:"return")).unwrap();


        let has_expression = match self.peek(0).unwrap() {
                &TokenType::SYMBOL(ref s) if s == ";" => false,
                _ => true
            };

        if has_expression {
            self.compile_expression();
        } else {
            //every function in jack MUST return a value;
            self.write_push_constant(0);
        }

        self.eat(jack!(SYMBOL:";")).unwrap();
        self.write_return();
    }

    fn compile_expression(&mut self) {
        //START FROM HERE;
        self.compile_term();
        loop {
            let op = match self.eat(jack!(SYMBOL:"+", SYMBOL:"-", SYMBOL:"*", SYMBOL:"/",
                        SYMBOL:"&", SYMBOL:"|", SYMBOL:">", SYMBOL:"<", SYMBOL:"=")) {
                            Some(s) => {
                                match s {
                                    TokenType::SYMBOL(o) => {
                                        o
                                    },
                                    _ => panic!()
                                }
                            },
                            None => break
                    };
            
            self.compile_term();

            self.write_bin_arthimetic(&op);
        }
    }

    //https://stackoverflow.com/questions/42075409/drop-a-immutable-borrow-to-make-a-mutable-borrow
    fn compile_term(&mut self) {
        //the eat function can not fit the needs

        let peek_token = self.peek(0).unwrap().clone();

        match peek_token {
            TokenType::IDENTIFIER(ref varname) => {
                //VARNAME | VARNAME [expression] | subroutine call
                let peek_token1 = self.peek(1).unwrap().clone();
                match peek_token1 {
                    TokenType::SYMBOL(ref s) if s == "[" => {
                        self.eat_force();
                        self.eat_force();
                        let entry = self.symbol_table.kind_of(varname).unwrap().clone();
                        //push arr
                        //push expression
                        //arr[expression]
                        //this is the array[expression]'s address
                        self.write_push(&entry);
                        self.compile_expression();
                        self.write_bin_arthimetic("+");
                        //put the array value into stack
                        //pop pointer 0"
                        //push that 0"
                        self.write_push_array_value();

                        self.eat(jack!(SYMBOL:"]")).unwrap();
                    },
                    TokenType::SYMBOL(ref s) if (s == "(" || s == ".") => {
                        self.compile_subroutine_call();
                    }
                    //normal variable
                    _ => {
                        //writeln!(self.target_file, "{}", peek_token);
                        let var = self.symbol_table.kind_of(varname).unwrap().clone();
                        self.write_push(&var);
                        self.eat_force();
                    }
                }
            },
            TokenType::SYMBOL(ref s) if (s == "-" || s == "~") => {
                self.eat_force();
                self.compile_term();
                self.write_uni_arthimetic(s);
            },
            TokenType::INTEGER(s) => {
                self.write_push_constant(s as u32);
                self.eat_force();
            }
            TokenType::SYMBOL(ref s) if s == "(" => {
                self.eat_force();
                self.compile_expression();
                self.eat(jack!(SYMBOL:")")).unwrap();
            },
            TokenType::KEYWORD(ref s) if (s == "true" || s == "false" || s == "null" || s == "this") => {
                match s.as_ref() {
                    "true" => { self.write_push_constant(0); self.write_uni_arthimetic("~")},
                    "false" => self.write_push_constant(0),
                    "null" => self.write_push_constant(0),
                    "this" => self.write_push_this(),
                    _ => panic!("never happen")
                }
                self.eat_force();
            },
            TokenType::STRING(ref s) => {
                self.write_push_constant(s.len() as u32);
                self.write_call("String.new", 1);
                for c in s.as_bytes() {
                    self.write_push_constant(*c as u32);
                    self.write_call("String.appendChar", 2);
                }
                self.eat_force();
            }
            _ => {
                panic!("failed to parse");
            }
        } 
    }

    fn compile_expression_list(&mut self) -> u32 {

        let mut nArgs :u32 = 0;
        let is_expression_list_empty = match self.peek(0).unwrap() {
            &TokenType::SYMBOL(ref s) if s == ")" => true,
            _ => false
        };

        if is_expression_list_empty {
            return nArgs;
        }

        self.compile_expression();
        nArgs += 1;
        //,
        loop {
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(s) => s,
                None => {break}
            };
            //expression
            self.compile_expression();
            nArgs += 1;
        }
        return nArgs;
    }

    fn compile_subroutine_call(&mut self) {

        //could be classname or instance name, We can find instance from
        //symbol table;
        let mut var_info = SymbolEntry::default();
        let is_method :bool;
        let mut function_name = String::default();

        let identifier = self.eat(jack!(IDENTIFIER:"")).unwrap();
        match identifier {
            TokenType::IDENTIFIER(ref s) => {
                match self.symbol_table.kind_of(s) {
                    Some(symbol_info) => {
                        var_info = symbol_info.clone();
                        is_method = true;
                        function_name.push_str(s);
                    },
                    _ => {
                        function_name.push_str(s);
                        is_method = false;
                    }
                }
            },
            _ => panic!("never to this")
        };

        //next could be . or (
        let valid_token = self.eat(jack!(SYMBOL:".", SYMBOL:"(")).unwrap();

        match valid_token {
            TokenType::SYMBOL(ref s) if s == "(" => {
                //this must be a method in current class
                let function_name = format!("{}.{}", self.class_name, function_name);
                self.write_push_this();
                let nArgs = self.compile_expression_list();
                self.write_call(&function_name, nArgs + 1);
                self.eat(jack!(SYMBOL:")")).unwrap();
            },
            TokenType::SYMBOL(ref s) if s == "." => {
                let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
                match valid_token {
                    TokenType::IDENTIFIER(ref s) => {
                        if is_method {
                            function_name = format!("{}.{}", var_info.var_type, s);
                        } else {
                            function_name.push('.');
                            function_name.push_str(s);
                        }
                    }
                    _ => panic!()
                }

                self.eat(jack!(SYMBOL:"(")).unwrap();

                let mut nArgs = 0; 
                if is_method {
                    //push var's this pointer to stack as argument 0
                    self.write_push(&var_info);
                    nArgs = self.compile_expression_list();
                    nArgs += 1;
                } else {
                    nArgs = self.compile_expression_list();
                }

                self.write_call(&function_name, nArgs);

                self.eat(jack!(SYMBOL:")")).unwrap();
            },
            _ => panic!("never will be here")
        }
    }

}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: ./jackanalyzier yourfile.jack");
        process::exit(-1);
    }

    let filename : &str = &args[1];

    //is args[1] a directory;

    let mut is_diretory :bool = false;


    let meta = fs::metadata(filename).unwrap_or_else(|err|{
        panic!("Problem parsing arguments: {}", err);
    });

    if meta.is_dir() {
        is_diretory = true;
    }

    let mut jack_files :Vec<String> = Vec::new();

    if is_diretory {
        let path_iter = fs::read_dir(filename).unwrap();
        for p in path_iter {
            let p = p.unwrap().path();
            let filename = p.to_str().unwrap();
            if filename.ends_with(".jack") {
                jack_files.push(filename.to_string());
            }
        }
    } else {
        jack_files.push(filename.to_string());
    }

    for filename in &jack_files {
        let mut jt = JackTokenizer::new(filename);
        jt.process();
        let mut je = JackAnalyzer::new(filename, jt);
        je.compile_class();
    }
}
