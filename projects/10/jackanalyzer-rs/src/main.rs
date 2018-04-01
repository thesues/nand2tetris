
extern crate regex;

use std::env;
use std::vec::Vec;
use std::process;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};
use std::fs::OpenOptions;
use std::fmt;

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
        let target_file_name = format!("{}MYT.{}", base_name(&self.filename), "xml");

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
    tokenizer: JackTokenizer
}

impl JackAnalyzer {
    pub fn new(jackfilename: &str, tokenizer :JackTokenizer ) -> JackAnalyzer {
        let target_file_name = format!("{}MYJ.{}", base_name(jackfilename), "xml");
        let target_file = OpenOptions::new()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(&target_file_name).unwrap();
        return JackAnalyzer{target_file_name: target_file_name, target_file: target_file, tokenizer:tokenizer};
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

        println!("<class>");
        //need macro to simplifiy the code
        let valid_token = self.eat(jack!(KEYWORD:"class")).unwrap();
        println!("{}", valid_token);

        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //let valid_token = self.eat(vec![TokenType::SYMBOL("{".to_string())]).unwrap();
        let valid_token = self.eat(jack!(SYMBOL:"{")).unwrap();
        println!("{}", valid_token);

        //0 or * class var declares
        while self.compile_class_var_dec(){};


        //0 or * class subroutine

        while self.compile_subroutine(){};

        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();
        println!("{}", valid_token);
        println!("</class>");
    }

    //  classVarDec*
    //  if return false, means no parser,
    // if return true, means could have more to parse
    fn compile_class_var_dec(&mut self) -> bool{
        //static | field
        let valid_token = match self.eat(jack!(KEYWORD:"static", KEYWORD:"field")) {
            Some(valid_token) => valid_token,
            None => {return false}
        };
        println!("<classVarDec>");
        println!("{}", valid_token);

        //type
        let valid_token = self.eat(jack!(KEYWORD:"int", KEYWORD:"char", KEYWORD:"boolean", IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //varname
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //(, varname)*
        loop {
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(valid_token) => valid_token,
                None => {break;}
            }; 
            println!("{}", valid_token);

            let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
            println!("{}", valid_token);
        }

        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();
        println!("{}", valid_token);

        println!("</classVarDec>");
        true
    }

    fn compile_subroutine(&mut self) -> bool{
        //constructor, function, method
        let valid_token = match self.eat(jack!(KEYWORD:"constructor", KEYWORD:"function", KEYWORD:"method")) {
            Some(valid_token) => valid_token,
            None => {return false}
        };
        println!("<subroutineDec>");
        println!("{}", valid_token);

        //void , type => int, boolean, char, ident
        let valid_token = self.eat(jack!(KEYWORD:"void", KEYWORD:"int", KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //subroutine name
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //symbol '('
        let valid_token = self.eat(jack!(SYMBOL:"(")).unwrap();
        println!("{}", valid_token);

        //one parameter list or none
        self.compile_parameter_list();

        //symbol ')'
        let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();
        println!("{}", valid_token);

        self.compile_subroutine_body();

        println!("</subroutineDec>");
        true
    }

    fn compile_subroutine_body(&mut self) {
        println!("{}", "<subroutineBody>");
        //symbol '{'
        let valid_token = self.eat(jack!(SYMBOL:"{")).unwrap();
        println!("{}", valid_token);


        while self.compile_var_dec() {}

        self.compile_statements();

        //symbol '}'
        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();
        println!("{}", valid_token);
        println!("{}", "</subroutineBody>");

    }
    fn compile_parameter_list(&mut self) {
        //type
        println!("<parameterList>");
        let valid_token = match self.eat(jack!(KEYWORD:"void", KEYWORD:"int", 
                                         KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")) { 
            Some(valid_token) => valid_token,
            None => {
                println!("</parameterList>");
                return;
            }
        };
        println!("{}", valid_token);

        //varname
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //(, type varname)
        loop {
            //,
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(valid_token) => valid_token,
                None => {break;}
            };
            println!("{}", valid_token);

            //type
            let valid_token = self.eat(jack!(KEYWORD:"void", KEYWORD:"int",
                                            KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")).unwrap();
            println!("{}", valid_token);

            let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
            println!("{}", valid_token);
        }
        println!("</parameterList>");

    }

    fn compile_var_dec(&mut self) -> bool{
        //var
        let valid_token = match self.eat(jack!(KEYWORD:"var")) {
            Some(valid_token) => valid_token,
            None => {return false;}
        };
        println!("{}", "<varDec>");
        println!("{}", valid_token);

        //type
        let valid_token = self.eat(jack!(KEYWORD:"void", KEYWORD:"int",
                                        KEYWORD:"boolean", KEYWORD:"char", IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //varName
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        loop {
            //,
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(valid_token) => valid_token,
                None => {break;}
            };
            println!("{}", valid_token);

            //varname
            let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
            println!("{}", valid_token);
        }

        //;
        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();
        println!("{}", valid_token);

        println!("{}", "</varDec>");
        true
    }

    fn compile_statements(&mut self ) {

        println!("{}","<statements>");

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

        println!("{}","</statements>");
    }

    fn compile_if_statement(&mut self) {


        println!("{}", "<ifStatement>");
        let valid_token = self.eat(jack!(KEYWORD:"if")).unwrap();
        println!("{}", valid_token);

        let valid_token = self.eat(jack!(SYMBOL:"(")).unwrap();
        println!("{}", valid_token);


        self.compile_expression();

        let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();
        println!("{}", valid_token);

        let valid_token = self.eat(jack!(SYMBOL:"{")).unwrap();
        println!("{}", valid_token);

        self.compile_statements();

        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();
        println!("{}", valid_token);


        //does it have else?
        match self.eat(jack!(KEYWORD:"else")) {
            None => {
                println!("{}", "</ifStatement>");
                return
            },
            Some(valid_token) => {
                println!("{}", valid_token)
            }
        }

        let valid_token = self.eat(jack!(SYMBOL:"{")).unwrap();
        println!("{}", valid_token);

        self.compile_statements();

        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();
        println!("{}", valid_token);
        println!("{}", "</ifStatement>");

    }

    fn compile_while_statement(&mut self) {
        println!("{}", "<whileStatement>");

        let valid_token = self.eat(jack!(KEYWORD:"while")).unwrap();
        println!("{}", valid_token);

        let valid_token = self.eat(jack!(SYMBOL:"(")).unwrap();
        println!("{}", valid_token);

        self.compile_expression();

        let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();
        println!("{}", valid_token);

        let valid_token = self.eat(jack!(SYMBOL:"{")).unwrap();
        println!("{}", valid_token);

        self.compile_statements();

        let valid_token = self.eat(jack!(SYMBOL:"}")).unwrap();
        println!("{}", valid_token);
        println!("{}", "</whileStatement>");

    }

    fn compile_do_statement(&mut self) {
        println!("{}", "<doStatement>");

        //do
        let valid_token = self.eat(jack!(KEYWORD:"do")).unwrap();
        println!("{}", valid_token);

        self.compile_subroutine_call();

        //;
        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();
        println!("{}", valid_token);

        println!("{}", "</doStatement>");

    }

    fn compile_let_statement(&mut self) {
        println!("{}", "<letStatement>");

        //let
        let valid_token = self.eat(jack!(KEYWORD:"let")).unwrap();
        println!("{}", valid_token);


        //ident
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);


        match self.eat(jack!(SYMBOL:"[")) {
            Some(valid_token) => {
                //[
                println!("{}", valid_token);

                self.compile_expression();

                //]
                let valid_token = self.eat(jack!(SYMBOL:"]")).unwrap();
                println!("{}", valid_token);
            },
            None=>{}
        }

        //=
        let valid_token = self.eat(jack!(SYMBOL:"=")).unwrap();
        println!("{}", valid_token);


        self.compile_expression();

        //;
        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();
        println!("{}", valid_token);

        println!("{}", "</letStatement>")
    }

    fn compile_return_statement(&mut self) {
        //return
        println!("{}", "<returnStatement>");

        let valid_token = self.eat(jack!(KEYWORD:"return")).unwrap();
        println!("{}", valid_token);



        let has_expression = match self.peek(0).unwrap() {
                &TokenType::SYMBOL(ref s) if s == ";" => false,
                _ => true
            };

        if has_expression {
            self.compile_expression();
        }

        let valid_token = self.eat(jack!(SYMBOL:";")).unwrap();
        println!("{}", valid_token);

        println!("{}", "</returnStatement>");
    }

    fn compile_expression(&mut self) {
        //START FROM HERE;
        println!("{}","<expression>");
        self.compile_term();
        loop {
            match self.eat(jack!(SYMBOL:"+", SYMBOL:"-", SYMBOL:"*", SYMBOL:"/",
                        SYMBOL:"&", SYMBOL:"|", SYMBOL:">", SYMBOL:"<", SYMBOL:"=")) {
                            Some(ref s) => println!("{}", s),
                            None => break
                        }
            
            self.compile_term();
        }
        println!("{}","</expression>");
    }

    //https://stackoverflow.com/questions/42075409/drop-a-immutable-borrow-to-make-a-mutable-borrow
    fn compile_term(&mut self) {
        //the eat function can not fit the needs
        println!("{}", "<term>");

        let peek_token = self.peek(0).unwrap().clone();

        match peek_token {
            TokenType::IDENTIFIER(ref _s) => {
                //VARNAME | VARNAME [expression] | subroutine call
                let peek_token1 = self.peek(1).unwrap().clone();
                match peek_token1 {
                    TokenType::SYMBOL(ref s) if s == "[" => {
                        println!("{}", peek_token);
                        self.eat_force();
                        println!("{}", peek_token1);
                        self.eat_force();
                        self.compile_expression();
                        let valid_token = self.eat(jack!(SYMBOL:"]")).unwrap();
                        println!("{}", valid_token);
                    },
                    TokenType::SYMBOL(ref s) if (s == "(" || s == ".") => {
                        self.compile_subroutine_call();
                    }
                    _ => {
                        println!("{}", peek_token);
                        self.eat_force();
                    }
                }
            },
            TokenType::SYMBOL(ref s) if (s == "-" || s == "~") => {
                println!("{}", peek_token);
                self.eat_force();
                self.compile_term();
            },
            TokenType::INTEGER(ref s) => {
                println!("{}", peek_token);
                self.eat_force();
            }
            TokenType::SYMBOL(ref s) if s == "(" => {
                println!("{}", peek_token);
                self.eat_force();
                self.compile_expression();
                let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();
                println!("{}", valid_token);
            },
            TokenType::KEYWORD(ref s ) if (s == "true" || s == "false" || s == "null" || s == "this") => {
                println!("{}", peek_token);
                self.eat_force();
            },
            TokenType::STRING(ref _s) => {
                println!("{}", peek_token);
                self.eat_force();
            }
            _ => {
                panic!("failed to parse");
            }
        } 
        println!("{}", "</term>");
    }

    fn compile_expression_list(&mut self) {

        println!("{}", "<expressionList>");


        let is_expression_list_empty = match self.peek(0).unwrap() {
            &TokenType::SYMBOL(ref s) if s == ")" => true,
            _ => false
        };

        if is_expression_list_empty {
            println!("{}", "</expressionList>");
            return;
        }

        self.compile_expression();
        //,
        loop {
            let valid_token = match self.eat(jack!(SYMBOL:",")) {
                Some(s) => s,
                None => {break}
            };
            println!("{}", valid_token);
            //expression
            self.compile_expression();
        }

        println!("{}", "</expressionList>");
    }

    fn compile_subroutine_call(&mut self) {
        let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
        println!("{}", valid_token);

        //next could be . or (
        let valid_token = self.eat(jack!(SYMBOL:".", SYMBOL:"(")).unwrap();

        match valid_token {
            TokenType::SYMBOL(ref s) if s == "(" => {
                println!("{}", valid_token);
                self.compile_expression_list();
                let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();
                println!("{}", valid_token);

            },
            TokenType::SYMBOL(ref s) if s == "." => {
                println!("{}", valid_token);

                let valid_token = self.eat(jack!(IDENTIFIER:"")).unwrap();
                println!("{}", valid_token);

                let valid_token = self.eat(jack!(SYMBOL:"(")).unwrap();
                println!("{}", valid_token);

                self.compile_expression_list();

                let valid_token = self.eat(jack!(SYMBOL:")")).unwrap();
                println!("{}", valid_token);
            },
            _ => {panic!("never will be herer")}
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
        jt.output();
        let mut je = JackAnalyzer::new(filename, jt);
        je.compile_class();
    }
}
