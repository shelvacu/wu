extern crate colored;
use colored::Colorize;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;
use wu::codegen::*;

use std::fs;
use std::fs::File;
use std::fs::metadata;

use std::env;
use std::path::Path;

use std::io::prelude::*;

pub fn path_ast(path: &str) -> Option<Vec<Statement>> {
    let source = match file_content(path) {
        Some(source) => source,
        None         => return None,
    };
    
    let lines = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, &path);
    
    match parser.parse() {
        Ok(ast) => Some(ast),
        Err(e)  => {
            e.display(&lines, &path);
            None
        }
    }
}

fn compile_path(path: &str) {
    let meta = metadata(path).unwrap();

    if meta.is_file() {
        match file_content(path) {
            Some(n) => write(path, &n),
            None    => (),
        }
    } else {
        let paths = fs::read_dir(path).unwrap();

        for path in paths {
            let path = format!("{}", path.unwrap().path().display());
            let split: Vec<&str> = path.split(".").collect();

            match split.get(1) {
                Some(n) if *n == "wu" => (),
                _ => continue,
            }

            compile_path(&format!("{}", path))
        }
    }
}

fn file_content(path: &str) -> Option<String> {
    let display = Path::new(path).display();
    
    let mut file = match File::open(&path) {
        Err(why) => panic!("failed to open {}: {}", display, why),
        Ok(file) => file,
    };

    let mut s = String::new();

    match file.read_to_string(&mut s) {
        Err(why) => panic!("failed to read {}: {}", display, why),
        Ok(_)    => compile(&s, path),
    }
}

fn write(path: &str, data: &str) {
    let path = Path::new(path);
    println!("{} {}", "compiled".green().bold(), path.display());

    let split_name = path.file_name().unwrap().to_str().unwrap().split(".");
    let split: Vec<&str> = split_name.collect();
    
    let parent_path = match path.parent() {
        Some(p) => match p.file_name() {
            Some(path) => path.to_str().unwrap(),
            None       => ".",
        },
        None => ".",
    };

    let output_name = format!("{}/{}.lua", parent_path, split.get(0).unwrap());

    let mut output_file = File::create(output_name).unwrap();
    match output_file.write_all(data.as_bytes()) {
        Ok(_)    => (),
        Err(why) => println!("{}", why)
    }
}

fn compile(source: &str, path: &str) -> Option<String> {    
    let lines = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, &path);

    match parser.parse() {
        Ok(ast)       => {
            let mut visitor = Visitor::new(&ast, &lines, &path);

            match visitor.validate() {
                Ok(_)         => {
                    let codegen = Codegen::new(&ast);

                    return Some(format!("{}", codegen))
                },

                Err(response) => response.display(&lines, &path),
            }
        },

        Err(response) => response.display(&lines, &path),
    }
    
    None
}

fn main() {
    match env::args().nth(1) {
        Some(a) => compile_path(&a),
        None    => println!("\
wu's transpiler

usage:
    wu <file>
    wu <folder>
        "),
    }
}

fn test() {
    let source = r#"

"#;
    let path = "test.wu";

    let lines = source.lines().map(|x| x.to_string()).collect();
    let lexer = make_lexer(source.clone().chars().collect(), &lines, &path);

    let mut parser = Parser::new(lexer.collect::<Vec<Token>>(), &lines, &path);

    match parser.parse() {
        Ok(ast)       => {
            println!("{:#?}", ast);

            let mut visitor = Visitor::new(&ast, &lines, &path);

            match visitor.validate() {
                Ok(_)         => {
                    let codegen = Codegen::new(&ast);

                    println!("```lua\n{}```", codegen)
                },
                Err(response) => response.display(&lines, &path),
            }
        },

        Err(response) => response.display(&lines, &path),
    }
}
