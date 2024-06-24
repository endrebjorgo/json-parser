use std::{env, fs};
use std::process::exit;
use std::collections::HashMap;
use std::str::FromStr;

type Result<T> = std::result::Result<T, ()>;

fn tokenize(bytes: Vec<u8>) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut curr_token = Vec::new();

    for b in bytes.iter() {
        let c = char::from_u32(*b as u32).unwrap();
        match c {
            '\r' | '\n' | '\t' => {
                if !curr_token.is_empty() {
                    let token = curr_token.iter().collect::<String>();
                    tokens.push(String::from(token.trim()));
                    curr_token.clear();
                }
            },
            '{' | '}' | '[' | ']' | ':' | ',' | '"' => {
                match curr_token.iter().last() {
                    Some('\\') | None => {
                        tokens.push(String::from(c));
                    },
                    Some(_) => {
                        let token = curr_token.iter().collect::<String>();
                        tokens.push(String::from(token.trim()));
                        curr_token.clear();
                        tokens.push(String::from(c));
                    },
                }
            },
            _ => curr_token.push(c),
        }
    }
    tokens.retain(|t| !&t.is_empty());
    return tokens;
}

/*
fn parse(tokens: &Vec<String>) -> Value {
    let mut values: Vec<Value> = Vec::new();
    let mut tokens_iter = tokens.iter();

    for t in tokens_iter {
        match t.as_str() {
            "{" => {
                println!("{}", &tokens_iter.next().unwrap());
            },
            "}" => {
                println!("Checking");
            },
            "[" => {
                println!("Checking");
            },
            "]" => {
                println!("Checking");
            },
            ":" => {
                println!("Checking");
            },
            "," => {
                println!("Checking");
            },
            "true" => {
                values.push(Value::True);
                println!("Checking");
            },
            "false" => {
                values.push(Value::False);
                println!("Checking");
            },
            "null" => {
                values.push(Value::Null);
                println!("Checking");
            },
            "\"" => {
                values.push(Value::Str("hi".to_string()));
                println!("Checking");
            },
            _ => {
                // Naive solution for now
                let token_result = i32::from_str(t); 
                match token_result {
                    Ok(i) => values.push(Value::Num(i)),
                    Err(_) => values.push(Value::Str(t.to_string())),
                }
            },
        }
    }
    return Value::Str(String::from("Hello"));

}
*/

#[derive(Debug)]
enum Value {
    Str(String),
    Num(i32),
    Obj(HashMap<String, Value>),
    Arr(Vec<Value>),
    True,
    False,
    Null,
}

fn main() -> Result<()> {
    let argv = env::args().collect::<Vec<String>>();
    let argc = argv.len();

    if argc != 2 {
        eprintln!("ERROR: please supply a file path as argument");
        exit(1);
    }

    let file_path = &argv[1];
    let bytes = fs::read(file_path).map_err(|err| {
        eprintln!("ERROR: could not read file {file_path}: {err}");
        exit(1);
    })?;

    let tokens = tokenize(bytes);
    for t in tokens.iter() {
        println!("{}", t);
    }

    // let result = parse(&tokens);

    Ok(())
}
