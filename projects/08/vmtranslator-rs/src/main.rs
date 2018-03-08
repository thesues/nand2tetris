use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader};
use std::process;
use std::fs;
use std::fs::OpenOptions;
use std::vec::Vec;
use std::fmt;
use std::collections::HashMap;



//TODO
fn base_name(filename: &str) -> &str {
    match filename.rfind(".") {
        Some(pos) => &filename[0..pos],
        None =>  "tempfile.asm"
    }
}

fn stem_name(file: &str) -> &str {
    match file.rfind("/") {
        Some(pos) => &file[(pos+1)..],
        None => file
    }
}


enum CommandType {
    C_ARITHMETIC,
    C_PUSH,
    C_POP,
    C_LABEL,
    C_GOTO,
    C_IF,
    C_FUNCTION,
    C_RETURN,
    C_CALL
}

struct Command {
    command_type: CommandType,
    arg1: String,
    arg2: u32,
    filename: String
}

impl fmt::Display for Command{
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = match self.command_type {
            CommandType::C_ARITHMETIC => String::from("arithmetic"),
            CommandType::C_PUSH => String::from("push"),
            CommandType::C_POP  => String::from("pop"),
            CommandType::C_LABEL => String::from("label"),
            CommandType::C_GOTO => String::from("goto"),
            CommandType::C_IF => String::from("if"),
            CommandType::C_FUNCTION => String::from("function"),
            CommandType::C_RETURN => String::from("return"),
            CommandType::C_CALL => String::from("call")
        };
        write!(f, "({}: {}, {}, {})", self.filename, display, self.arg1, self.arg2)
    }
}


struct VMParser<'a> {
    commands: Vec<Command>,
    currentPos :usize,
    file: &'a File,
    filename: &'a str,
}


impl<'a> VMParser<'a> {

    pub fn new(file: &'a File, filename: &'a str) -> VMParser<'a> {
        VMParser{commands:Vec::new(), currentPos:0, file: file, filename:filename}
    }

    pub fn process(&mut self) {
        let mut line_num :u32 = 0;
        for line in BufReader::new(self.file).lines() {
            line_num += 1;
            //skip empty line
            let s= line.unwrap();
            if s.is_empty() {
                continue;
            }

            //skip all the comment
            let comment_offset = s.find("//").unwrap_or(s.len());
            let (first, _last) = s.split_at(comment_offset);
            if first.is_empty() {
                continue;
            }
            //DEBUG
            //println!("{}", first);

            let mut iter = first.split_whitespace();

            let cmd :CommandType;
            let mut arg1 :String = String::from("");
            let mut arg2 :u32 = 0;

            //command
            if let Some(c) = iter.next() {
                    cmd = match c {
                            "push" => CommandType::C_PUSH,
                            "pop"  => CommandType::C_POP,
                            "add"| "sub" | "neg" | "eq" | "gt" | "and" | "or" | "not" | "lt" => {
                                arg1 = String::from(c);
                                CommandType::C_ARITHMETIC
                            },
                            "goto" => CommandType::C_GOTO,
                            "if-goto" => CommandType::C_IF,
                            "label"  => CommandType::C_LABEL,
                            "function" => CommandType::C_FUNCTION,
                            "return" => CommandType::C_RETURN,
                            "call" => CommandType::C_CALL,
                            _ => {
                                panic!("do not recognize this command: {}", c);
                            }
                    }
            } else {
                    panic!("do not have commands: Failed to parser the vm file at line {}, {}", self.filename, line_num);
            }
            //arg1
            //FIXME: should have more clear syntax
            if let Some(arg) =  iter.next() {
                match cmd {
                    CommandType::C_CALL | CommandType::C_FUNCTION | 
                    CommandType::C_IF | CommandType::C_LABEL | CommandType::C_PUSH | CommandType::C_POP | CommandType::C_GOTO=> {
                        arg1 = String::from(arg);
                    },
                    CommandType::C_ARITHMETIC|CommandType::C_RETURN => {
                        arg1 = String::from("");
                    }
                }
            } else {
                match cmd {
                    CommandType::C_CALL | CommandType::C_FUNCTION | 
                    CommandType::C_IF | CommandType::C_LABEL | CommandType::C_PUSH | CommandType::C_POP=> {
                            println!("No arg1 : Failed to parser the vm file at line {}, {}", self.filename, line_num);
                            panic!();
                    },
                    _ => {}
                }
            }

            //arg2
            if let Some(arg) = iter.next() {
                match cmd {
                    CommandType::C_PUSH | CommandType::C_POP | CommandType::C_CALL | CommandType::C_FUNCTION => {
                        arg2 = arg.parse::<u32>().unwrap()
                    },
                    _ => {
                        arg2 = 0;
                    }
                }
            } else {
                match cmd {
                    CommandType::C_PUSH | CommandType::C_POP | CommandType::C_CALL | CommandType::C_FUNCTION=> {
                        println!("No arg2 : Failed to parser the vm file at line {}, {}", self.filename, line_num);
                        panic!();
                    },
                    _ => {}
                }
            }

            let parsed_command = Command{filename: String::from(self.filename), command_type:cmd, arg1, arg2};

            self.commands.push(parsed_command);
        }
    }

    pub fn has_more_commands(&self) -> bool {
        self.currentPos < self.commands.len()
    }

    pub fn advance(&mut self) -> &Command {
        let temp = self.currentPos;
        self.currentPos += 1;
        return &self.commands[temp];
    }
}



//TODO: these asm could be build shorter

const PUSH_ASM: &'static str = ("
    @{{i}}
    D=A
    @{{SEGMENT}}
    D=M+D
    @R13
    M=D
    @R13
    A=M
    D=M
    @SP
    A=M
    M=D
    @SP
    M=M+1");

const POP_ASM: &'static str = ("
    @{{i}}
    D=A
    @{{SEGMENT}}
    D=M+D
    @R13
    M=D
    @SP
    M=M-1
    @SP
    A=M
    D=M
    @R13
    A=M
    M=D");

const CONSTANT_PUSH_ASM: &'static str = ("
    @{{i}}
    D=A
    @SP
    A=M
    M=D
    @SP
    M=M+1");

const FIXEDSEGMENT_PUSH_ASM: &'static str = ("
    @{{VARABLE}}
    D=M
    @SP
    A=M
    M=D
    @SP
    M=M+1");


/*
this could be:
@SP
AM=M-1
D=M
@{{VARABLE}}
M=D\n
*/
const FIXEDSEGMENT_POP_ASM: &'static str = ("
    @SP
    M=M-1
    @SP
    A=M
    D=M
    @{{VARABLE}}
    M=D\n
");

const ARITHMETIC_ASM: &'static str = ("
    @SP
    M=M-1
    @SP
    A=M
    D=M
    @R13
    M=D
    @SP
    M=M-1
    @SP
    A=M
    D=M
    @R13
    D=D{{operator}}M
    @SP
    A=M
    M=D
    @SP
    M=M+1");

const ARITHMETIC_NOT_NEG_ASM: &'static str = ("
    @SP
    M=M-1
    @SP
    A=M
    D={{operator}}M
    @SP
    A=M
    M=D
    @SP
    M=M+1");

const ARITHMETIC_CMP_ASM: &'static str = ("
    @SP
    M=M-1
    @SP
    A=M
    D=M
    @R13
    M=D
    @SP
    M=M-1
    @SP
    A=M
    D=M
    @R13
    D=D-M
    @{{CMP}}.{{N}}
    D;{{CMP}}
    @SP
    A=M
    M=0
    @{{CMP}}.END.{{N}}
    0;JMP
    ({{CMP}}.{{N}})
        @SP
        A=M
        M=-1
    ({{CMP}}.END.{{N}})
        @SP
        M=M+1");

const GOTO_ASM: &'static str ="
    @{{LABEL}}
    0;JMP
";


const IFGOTO_ASM: &'static str ="
    @SP
    AM=M-1
    D=M
    @{{LABEL}}
    D;JGT
";

const LABEL_ASM: &'static str = "
    ({{LABEL}})
";


const FUNCTION_ASM: &'static str = "
({{FILENAME}}.{{FUNCTION}})";

const FUNCTION_REPEAT_NARGS_ASM: &'static str= "
    @SP
    A=M
    M=0
    @SP
    M=M+1";


const RETURN_ASM : &'static str = "
    //R13 = *(LCL-5)
    @LCL
    D=M
    @5
    A=D-A
    D=M
    @R13
    M=D
    //*ARG = pop
    @SP
    A=M-1
    D=M
    @ARG
    A=M
    M=D
    //SP=ARG+1
    @ARG
    D=M+1
    @SP
    M=D
    //THAT=*(LCL-1)
    @LCL
    A=M-1
    D=M
    @THAT
    M=D
    //THIS = *(LCL-2)
    @LCL
    D=M
    @2
    A=D-A
    D=M
    @THIS
    M=D
    //ARG = *(LCL-3)
    @LCL
    D=M
    @3
    A=D-A
    D=M
    @ARG
    M=D
    //LCL = *(LCL-4)
    @LCL
    D=M
    @4
    A=D-A
    D=M
    @LCL
    M=D
    //goto retAddr
    @R13
    A=M
    0;JMP";

struct CodeWriter<'a> {
    target_file : &'a File,
    filename: String,
    literal_map :HashMap<String, String>,
    operator_map: HashMap<String, String>,
    //eq,gt,lt's translation needs a label to jump
    //each jump has an uniq lable, inscrease cmp_count every time when translating eq,gt,lt
    cmp_counter: u32,
    current_function: String
}

impl<'a> CodeWriter<'a> {
    fn new(target_file: &'a File, filename: String) -> CodeWriter {
        let mut cw = CodeWriter{target_file:target_file, filename:filename, 
                                literal_map: HashMap::new(),
                                operator_map: HashMap::new(),
                                cmp_counter:0,
                                current_function: String::from("")
                                };

        cw.literal_map.insert(String::from("this"), String::from("THIS"));
        cw.literal_map.insert(String::from("that"), String::from("THAT"));
        cw.literal_map.insert(String::from("argument"), String::from("ARG"));
        cw.literal_map.insert(String::from("local"), String::from("LCL"));

        //two args
        cw.operator_map.insert(String::from("add"), String::from("+"));
        cw.operator_map.insert(String::from("sub"), String::from("-"));
        cw.operator_map.insert(String::from("and"), String::from("&"));
        cw.operator_map.insert(String::from("or"), String::from("|"));

        //one args
        cw.operator_map.insert(String::from("neg"), String::from("-"));
        cw.operator_map.insert(String::from("not"), String::from("!"));

        return cw;
    }

    fn increase_counter(&mut self) -> u32 {
        let ret = self.cmp_counter;
        self.cmp_counter += 1;
        ret
    }

    fn write_asm(&mut self , cmd : &Command) {
        //static, this, local, argument, that, constant, pointer, temp
        let mut s :String = String::from("");
        match cmd.command_type {
            CommandType::C_PUSH => {
                match cmd.arg1.as_ref() {
                    "static" => {
                        //let static_variable_name = base_name(&self.filename).to_owned() + &cmd.arg2.to_string();
                        let local_filename : &str = stem_name(base_name(&self.filename));
                        let static_variable_name = format!("{}.{}", local_filename, cmd.arg2);

                        s = String::from(FIXEDSEGMENT_PUSH_ASM);
                        s= s.replace("{{VARABLE}}", &static_variable_name);
                    },
                    "constant" => {
                        s = String::from(CONSTANT_PUSH_ASM);
                        s = s.replace("{{i}}", &cmd.arg2.to_string());
                    },
                    "temp" => {
                        let temp_address = format!("{}", 5 + cmd.arg2);
                        s = String::from(FIXEDSEGMENT_PUSH_ASM);
                        s = s.replace("{{VARABLE}}", &temp_address);
                    },
                    "pointer" => {
                        let address = match cmd.arg2 {
                            0 => "THIS",
                            1 => "THAT",
                            _  => {
                                println!("arg2 should be 0 or 1");
                                panic!();
                            }
                        };
                        s = String::from(FIXEDSEGMENT_PUSH_ASM);
                        s = s.replace("{{VARABLE}}", &address);
                    },
                    //this, that, argument, local
                    _ => {
                        //process arg1
                        //Dereferencing strings and HashMaps in Rust, fuck this
                        s = String::from(PUSH_ASM);
                        let replace_name = self.literal_map.get(&cmd.arg1).unwrap();
                        s = s.replace("{{SEGMENT}}",replace_name);
                        //process arg2
                        s = s.replace("{{i}}", &cmd.arg2.to_string());
                    }
                };
            },
            CommandType::C_POP => {
                match cmd.arg1.as_ref() {
                    "static" => {
                        let local_filename : &str = stem_name(base_name(&self.filename));
                        let static_variable_name = format!("{}.{}", local_filename, cmd.arg2);

                        s = String::from(FIXEDSEGMENT_POP_ASM);
                        s= s.replace("{{VARABLE}}", &static_variable_name);
                    },
                    "temp" => {
                        let temp_address = format!("{}", 5 + cmd.arg2);
                        s = String::from(FIXEDSEGMENT_POP_ASM);
                        s = s.replace("{{VARABLE}}", &temp_address);
                    },
                    "pointer" => {
                        let address = match cmd.arg2 {
                            0 => "THIS",
                            1 => "THAT",
                            _  => {
                                println!("arg2 should be 0 or 1");
                                panic!();
                            }
                        };
                        s = String::from(FIXEDSEGMENT_POP_ASM);
                        s = s.replace("{{VARABLE}}", &address);
                    }
                    //this, that, argument, local
                    _ => {
                        s = String::from(POP_ASM);
                        let replace_name = self.literal_map.get(&cmd.arg1).unwrap();
                        s = s.replace("{{SEGMENT}}",replace_name);
                        //process arg2
                        s = s.replace("{{i}}", &cmd.arg2.to_string());

                    }
                }
            },
            CommandType::C_ARITHMETIC => {
                match cmd.arg1.as_ref() {
                    "sub"|"add"|"and"|"or" => {
                        s = String::from(ARITHMETIC_ASM);
                        s = s.replace("{{operator}}", self.operator_map.get(&cmd.arg1).unwrap());
                    },
                    "not"|"neg" => {
                        s = String::from(ARITHMETIC_NOT_NEG_ASM);
                        s = s.replace("{{operator}}", self.operator_map.get(&cmd.arg1).unwrap());

                    },
                    "eq"|"gt"|"lt" => {
                        s = String::from(ARITHMETIC_CMP_ASM);
                        //convert "eq"|"gt"|"lt"to "JEQ"| "JGT" | "JLT"
                        s = s.replace("{{CMP}}", &format!("J{}",&cmd.arg1.to_ascii_uppercase()));
                        let i = self.increase_counter();
                        let stemname =stem_name(base_name(&self.filename));
                        s = s.replace("{{N}}", &format!("{}.{}", &stemname, i));
                    },
                    _ => {
                        println!("only know how to deal with this command: {}", cmd.arg1);
                        panic!();
                    }

                }
            },
            CommandType::C_GOTO => {
                s = String::from(GOTO_ASM);
                s = s.replace("{{LABEL}}", &cmd.arg1);

            }, 
            CommandType::C_IF => {
                s = String::from(IFGOTO_ASM);
                s = s.replace("{{LABEL}}", &cmd.arg1);
            },
            CommandType::C_LABEL => {
                s = String::from(LABEL_ASM);
                let local_label = format!("{}{}", self.current_function, cmd.arg1);
                s = s.replace("{{LABEL}}", &local_label);
            },
            CommandType::C_FUNCTION => {
                s = String::from(FUNCTION_ASM);
                let local_filename = stem_name(base_name(&self.filename));
                s = s.replace("{{FILENAME}}", local_filename);
                s = s.replace("{{FUNCTION}}", &cmd.arg1);
                println!("{}", cmd.arg2);
                for i in 0..cmd.arg2 {
                    s.push_str(FUNCTION_REPEAT_NARGS_ASM);
                }
                self.current_function = cmd.arg1.to_string();
            },
            CommandType::C_CALL => {

            },
            CommandType::C_RETURN => {
                s = String::from(RETURN_ASM);
            }
        }

        self.target_file.write(s.as_bytes()).expect("failed to write to file");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("usage: ./vmtranslate yourfile.vm");
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

    let mut vm_files :Vec<(File, String)> = Vec::new();

    if is_diretory {
        let path_iter = fs::read_dir(filename).unwrap_or_else(|err|panic!("failed to open direcoty {}, {}", filename, err));
        for p in path_iter {
            let p = p.unwrap().path();
            let filename = p.to_str().unwrap();
            if filename.ends_with(".vm") {
                vm_files.push((File::open(filename).expect("failed to open a file"), String::from(filename)));
                println!("opend the file: {}", filename);
            }
        }
    } else {
        let f = File::open(filename).unwrap_or_else(|err|{
            panic!("Problem parsing arguments: {}", err);
        });
        vm_files.push((f, String::from(filename)));
        println!("opend the file: {}", filename);
    }


    let target_file_name = format!("{}.{}", base_name(filename), "asm");

    let target_file = OpenOptions::new()
                            .create(true)
                            .write(true)
                            .truncate(true)
                            .open(target_file_name).unwrap();

    for f in &vm_files {
        let fd = &f.0;
        let filename=&f.1;
        let mut parser = VMParser::new(fd, filename);
        parser.process();
        //output the all commands for one file;

        let mut code_writer = CodeWriter::new(&target_file, String::from(filename.as_ref()));
        while parser.has_more_commands() {
            let cmd = parser.advance();
            code_writer.write_asm(cmd);
        }
    }

//    parser.process(&vm_files);




    //iterate each vm command to translate into hack language
}
