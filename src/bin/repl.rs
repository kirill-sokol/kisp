use std::io;

use std::sync::Arc;
use linefeed::{Interface, ReadResult};

use kisp::lexer::Lexer;
use kisp::{interpreter, parser};
use kisp::value;

use kisp::scope::ScopeRef;
use kisp::value::EvalValue;

const HISTORY_FILE: &str = ".kisp-history";
fn main() -> io::Result<()>{
    let interface = Arc::new(Interface::new("REPL for Kirill's Lisp")?);
    println!("kisp - Kirill's Lisp. Multi-line input: end a line with \\");
    interface.set_prompt("kisp> ")?;
    let mut env: Option<ScopeRef> = None;

    if let Err(e) = interface.load_history(HISTORY_FILE) {
        if e.kind() == io::ErrorKind::NotFound {
            println!("History file {} doesn't exist, not loading history.", HISTORY_FILE);
        } else {
            eprintln!("Could not load history file {}: {}", HISTORY_FILE, e);
        }
    }

    let mut line_acc: String = String::new();
    while let ReadResult::Input(line) = interface.read_line()? {
        line_acc.push_str(line.as_str());
        if line.ends_with('\\') {
            continue;
        }
        line_acc = line_acc.replace("\\","\n");
        if !line_acc.trim().is_empty() {
            interface.add_history_unique(line_acc.clone());
            let (result, new_env) = do_line(env, line_acc);
            match result{
                Ok(v) => {
                    println!("{}", v);
                }
                Err(e) => {
                    println!("Err: {}", e);
                }
            }
            line_acc = String::new();
            env = new_env;
        }

    }
    interface.save_history(HISTORY_FILE)?;
    Ok(() )
}

fn do_line(env: Option<ScopeRef>, line: String) -> (Result<EvalValue, String>, Option<ScopeRef>) {
    let lexer = Lexer::from_text(line.as_str());
    let mut iter = lexer.into_iter();
    let ast = match parser::parse(&mut iter) {
        Ok(ast) => ast,
        Err(e) => return (Err(format!("Parser: {:?}", e)),env),
    };
    let (result, modded_env) = interpreter::eval(&ast, env);
    (
        result.map_err(|err| format!("Eval: {:?}", err)).map(|v|v.0),
        Some(modded_env)
    )

}