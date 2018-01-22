extern crate colored;

mod wu;
use wu::lexer::*;
use wu::parser::*;
use wu::visitor::*;
use wu::codegen::*;

fn main() {
    let source = r#"
fib :: {(a: int) int -> match a {
	| 0 -> 0
	| 1 -> 1
	| _ -> fib(a - 1) + fib(a - 2)
}}

foo: string = fib(10)
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
