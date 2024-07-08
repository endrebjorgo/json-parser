use std::{env, fs};
use std::process::exit;
use std::collections::HashMap;
use std::cell::Cell;
use std::path::Path;
use std::ffi::OsStr;
use std::str::FromStr;
use std::fmt;

#[derive(Debug)]
enum JSONValue {
    Obj(HashMap<String, JSONValue>),
    Arr(Vec<JSONValue>),
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
}

impl fmt::Display for JSONValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JSONValue::Obj(hm) => {
                write!(f, "{}", "{\n");
                for (key, value) in hm.iter() {
                    write!(f, "    {}: {},\n", key, value);
                }
                write!(f, "{}", "}")
            },
            JSONValue::Arr(v) => {
                write!(f, "[\n");
                for x in v.iter() {
                    write!(f, "    {},\n", x);
                }
                write!(f, "{}", "]")
            },
            JSONValue::Str(s) => write!(f, "\"{}\"", s.replace("\\", "\\\\")),
            JSONValue::Num(n) => write!(f, "{}", n),
            JSONValue::Bool(b) => write!(f, "{}", b),
            JSONValue::Null => write!(f, "{}", "null"),
            _ => unreachable!(),
        }
    }
}

fn parse_json(bytes: &Vec<u8>) -> JSONValue {
    let tokens = tokenize(bytes);
    let cursor = Cell::<usize>::new(0);
    return parse_value(&tokens, &cursor);
}

fn tokenize(bytes: &Vec<u8>) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut curr_token = Vec::new();
    let mut in_string = false;
    let mut escape = false;

    for b in bytes.iter() {
        let c = char::from_u32(*b as u32).unwrap();
        match c {
            '\n' | '\r' | '\t' => {
                // White space. These are ignored by the tokenizer, but mark the
                // end of the current token..
                assert!(!in_string);
                assert!(!escape);
                if !curr_token.is_empty() {
                    tokens.push(curr_token.iter().collect());
                    curr_token.clear();
                }
            },
            '{' | '}' | '[' | ']' | ':' | ',' => {
                // Special separator characters. Treated as normal characters in
                // strings, but become a single token outside of them.
                assert!(!escape);
                if in_string {
                    curr_token.push(c);
                } else {
                    if !curr_token.is_empty() {
                        tokens.push(curr_token.iter().collect());
                        curr_token.clear();
                    }
                    tokens.push(c.to_string());
                }
            },
            '\\' => {
                // Backslash. Must be in a string (if valid JSON) and marks that
                // a character will be escaped. Becomes '\' if already escaped.
                assert!(in_string);
                if escape {
                    curr_token.push(c);
                    escape = false;
                } else {
                    escape = true;
                }
            },
            '/' => {
                // Forward slash. Can be escaped.
                assert!(in_string);
                curr_token.push(c);
                escape = false; // Just in case the '/' was escaped.
            },
            'b' | 'f' | 'n' | 'r' | 't' => {
                // Escape characters in JSON strings. If in a string and 
                // preceded by a backslash, a character such as n becomes \n.
                if escape {
                    assert!(in_string);
                    match c {
                        'b' => curr_token.push(8u8 as char),
                        'f' => curr_token.push(12u8 as char),
                        'n' => curr_token.push('\n'),
                        'r' => curr_token.push('\r'),
                        't' => curr_token.push('\t'),
                        _ => unreachable!(),
                    }
                    escape = false;
                } else {
                    curr_token.push(c);
                }
            },
            'u' => {
                // Unicode character. TODO: Not implemented yet.
                curr_token.push(c);
                escape = false;
            },
            '\"' => {
                // Quotes. Signify the start/end of a string, but not if they 
                // are escaped inside of a string.
                if escape {
                    assert!(in_string);
                    curr_token.push(c);
                    escape = false;
                } else {
                    in_string = !in_string;
                    if !curr_token.is_empty() {
                        tokens.push(curr_token.iter().collect());
                        curr_token.clear();
                    }
                    tokens.push(c.to_string());
                }
            },
            ' ' => {
                // Spaces are ignored if they are not part of a string
                assert!(!escape);
                if in_string {
                    curr_token.push(c);
                } else {
                    if !curr_token.is_empty() {
                        tokens.push(curr_token.iter().collect());
                        curr_token.clear();
                    }
                }
            },
            _ => {
                // Other characters. These just combine into numbers/words/other
                assert!(!escape);
                curr_token.push(c);
            },
        }
    }
    assert!(curr_token.is_empty());
    return tokens;
}

fn parse_value(tokens: &Vec<String>, cursor: &Cell<usize>) -> JSONValue {
    match tokens[cursor.get()].as_str() {
        "{" => {
            // Parsing object
            cursor.set(cursor.get() + 1);
            return parse_object(&tokens, cursor);
        },
        "[" => {
            // Parsing array
            cursor.set(cursor.get() + 1);
            return parse_array(&tokens, cursor);
        },
        "\"" => {
            // Parsing string
            cursor.set(cursor.get() + 1);
            return JSONValue::Str(parse_string(&tokens, cursor));
        },
        "true" => {
            // Parsing true value
            cursor.set(cursor.get() + 1);
            return JSONValue::Bool(true);
        },
        "false" => {
            // Parsing false value
            cursor.set(cursor.get() + 1);
            return JSONValue::Bool(false);
        },
        "null" => {
            // Parsing null value
            cursor.set(cursor.get() + 1);
            return JSONValue::Null;
        },
        _ => {
            // Parsing number
            return parse_number(&tokens, cursor);
        },
    }
}

fn parse_object(tokens: &Vec<String>, cursor: &Cell<usize>) -> JSONValue {
    let mut hm = HashMap::<String, JSONValue>::new();
    loop {
        match tokens[cursor.get()].as_str() {
            "}" => {
                // Empty object
                cursor.set(cursor.get() + 1);
                break;
            },
            "\"" => {
                cursor.set(cursor.get() + 1);
                let mut key = parse_string(&tokens, &cursor);
                assert_eq!(tokens[cursor.get()], ":");
                cursor.set(cursor.get() + 1);
                let value = parse_value(&tokens, &cursor);
                hm.insert(key, value);
            },
            _ => unreachable!(), // Assuming valid JSON.
        }
        match tokens[cursor.get()].as_str() {
            "," => {
                // New entry
                cursor.set(cursor.get() + 1);
                continue;
            },
            "}" => {
                // End of object
                cursor.set(cursor.get() + 1);
                break;
            },
            x => {
                println!("{}", x);
                println!("{:?}", hm);
                println!("{:?}", &tokens[cursor.get()-2..cursor.get()+2]);
                unreachable!();
            }
        }
    }
    return JSONValue::Obj(hm);
}

fn parse_array(tokens: &Vec<String>, cursor: &Cell<usize>) -> JSONValue {
    let mut array = Vec::<JSONValue>::new();

    loop {
        match tokens[cursor.get()].as_str() {
            "]" => {
                // Empty array
                cursor.set(cursor.get() + 1);
                break;
            },
            _ => {
                let value = parse_value(&tokens, &cursor);
                array.push(value);
            }
        }

        match tokens[cursor.get()].as_str() {
            "," => {
                // New entry
                cursor.set(cursor.get() + 1);
                continue;
            },
            "]" => {
                // End of array 
                cursor.set(cursor.get() + 1);
                break;
            },
            y => {
                unreachable!();
            },
        }
    }
    return JSONValue::Arr(array);
}

fn parse_string(tokens: &Vec<String>, cursor: &Cell<usize>) -> String {
    let curr = &tokens[cursor.get()];
    cursor.set(cursor.get() + 1);
    let next = &tokens[cursor.get()];

    match next.as_str() {
        "\"" => {
            cursor.set(cursor.get() + 1);
            return String::from(curr);
        },
        _ => {
            return String::new();
        }
    }
}

fn parse_number(tokens: &Vec<String>, cursor: &Cell<usize>) -> JSONValue {
    let token = &tokens[cursor.get()];
    let number = f64::from_str(token).unwrap();
    cursor.set(cursor.get() + 1);
    return JSONValue::Num(number);
}

type Result<T> = std::result::Result<T, ()>;

fn main() -> Result<()> {
    let argv = env::args().collect::<Vec<String>>();
    let argc = argv.len();

    if argc != 2 {
        eprintln!("ERROR: please supply one argument being the file path");
        exit(1);
    }

    let file_path = Path::new(&argv[1]);
    assert_eq!(file_path.extension().and_then(OsStr::to_str), Some("json"));

    let bytes = fs::read(file_path).map_err(|err| {
        eprintln!("ERROR: could not read file {:?}: {err}", file_path);
        exit(1);
    })?;

    
    let res = tokenize(&bytes); 
    for (i, r) in res.iter().enumerate() {
        println!("Token {:03}: {}",i , r);
    }
    let x = parse_json(&bytes);
    println!("{}", x);
    //println!("{:?}", x);

    Ok(())
}

#[cfg(test)]
mod tests {
    mod parse_string {
        use super::super::{Cell, parse_string};
        #[test]
        fn test_parse_string_1() {
            let tokens = vec!["\"".to_string(), "hello".to_string(), "\"".to_string()];
            let cursor = Cell::new(1 as usize);
            let x = parse_string(&tokens, &cursor);
            assert_eq!(x, "hello".to_string());
        }

        #[test]
        fn test_parse_string_2() {
            let tokens = vec!["\"".to_string(), "".to_string(), "\"".to_string()];
            let cursor = Cell::new(1 as usize);
            let x = parse_string(&tokens, &cursor);
            assert_eq!(x, "".to_string());
        }

        #[test]
        fn test_parse_string_3() {
            let tokens = vec!["\"".to_string(), "\"".to_string(), "\"".to_string()];
            let cursor = Cell::new(1 as usize);
            let x = parse_string(&tokens, &cursor);
            assert_eq!(x, "\"".to_string());
        }
    }
}
