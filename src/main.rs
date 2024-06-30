use std::{env, fs};
use std::process::exit;
use std::collections::HashMap;
use std::cell::Cell;

type Result<T> = std::result::Result<T, ()>;

#[derive(Debug)]
enum JSONValue {
    Obj(HashMap<String, JSONValue>),
    Arr(Vec<JSONValue>),
    Str(String),
    Num(f64),
    Bool(bool),
    Null,
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
                // White space - just ignore these for now, but make them end 
                // the previous token
                assert!(!in_string);
                if !curr_token.is_empty() {
                    tokens.push(curr_token.iter().collect());
                    curr_token.clear();
                }
                escape = false;
            },
            '{' | '}' | '[' | ']' | ':' | ',' | '+' | '-' => {
                // Special characters. Treated as normal characters in string
                if in_string {
                    curr_token.push(c);
                } else {
                    if !curr_token.is_empty() {
                        tokens.push(curr_token.iter().collect());
                        curr_token.clear();
                    }
                    tokens.push(c.to_string());
                }
                escape = false;
            },
            '\\' => {
                // Backslash. Assumes correct JSON for now, so it must be in a 
                // string and signifies that a character is to be escaped.
                assert!(in_string);
                if escape {
                    curr_token.push(c);
                    escape = false;
                } else {
                    escape = true;
                }
            },
            '/' => {
                // Forward slash. CAN be escaped.
                curr_token.push(c);
                escape = false; // Just in case the '/' was escaped.
            },
            'b' | 'f' | 'n' | 'r' | 't' => {
                // Escapable characters in JSON strings. If in a string and 
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
                // Unicode character. Not implemented yet.
                curr_token.push(c);
                escape = false;
            },
            '\"' => {
                // Quotes. Signify the start/end of a string, but not if they 
                // are escaped inside of a string.
                
                // TODO: Problem arises when there is a one character string 
                // containing an escaped ". These are basically parsed the same 
                // as a quote. They need to be differentiated...
                
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
                if in_string {
                    curr_token.push(c);
                } else {
                    if !curr_token.is_empty() {
                        tokens.push(curr_token.iter().collect());
                        curr_token.clear();
                    }
                }
                escape = false;
            },
            _ => {
                // Other characters. These just combine into numbers/words/other
                curr_token.push(c);
                escape = false;
            },
        }
    }
    assert!(curr_token.is_empty());
    return tokens;
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
                let mut key = String::new();
                let s = &tokens[cursor.get()];
                match s.as_str() {
                    "\"" => {
                        cursor.set(cursor.get() + 1);
                    },
                    _ => {
                        cursor.set(cursor.get() + 1);
                        assert_eq!(tokens[cursor.get()].as_str(), "\"");
                        cursor.set(cursor.get() + 1);
                        key.push_str(s);
                    },
                }
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

fn parse_number(tokens: &Vec<String>, cursor: &Cell<usize>) -> JSONValue {
    return JSONValue::Num(1.0);
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
            let s = &tokens[cursor.get()];
            match s.as_str() {
                "\"" => {
                    cursor.set(cursor.get() + 1);
                    return JSONValue::Str(String::new());
                },
                _ => {
                    cursor.set(cursor.get() + 1);
                    assert_eq!(tokens[cursor.get()].as_str(), "\"");
                    cursor.set(cursor.get() + 1);
                    return JSONValue::Str(String::from(s));
                },
            }
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
        x => {
            // Parsing number (TODO: Is this true?)
            // Temporary workaround to not bother implementing numbers yet...
            let mut c = cursor.get();
            let forbidden_tokens = vec![",", "\"", "]", "}"];
            while !forbidden_tokens.contains(&tokens[c].as_str()) {
                c += 1;
            }
            cursor.set(c);
            return JSONValue::Num(1.0);
        },
    }
}

fn parse_json(bytes: &Vec<u8>) -> JSONValue {
    let tokens = tokenize(bytes);
    let cursor = Cell::<usize>::new(0);

    return parse_value(&tokens, &cursor);
}

fn main() -> Result<()> {
    let argv = env::args().collect::<Vec<String>>();
    let argc = argv.len();

    if argc != 2 {
        eprintln!("ERROR: please supply one argument being the file path");
        exit(1);
    }
    // TODO: Check if .json file extension.

    let file_path = &argv[1];
    let bytes = fs::read(file_path).map_err(|err| {
        eprintln!("ERROR: could not read file {file_path}: {err}");
        exit(1);
    })?;

    
    let res = tokenize(&bytes); 
    for (i, r) in res.iter().enumerate() {
        println!("Token {:03}: {}",i , r);
    }
    let x = parse_json(&bytes);
    println!("{:?}", x);

    Ok(())
}

/*
#[cfg(test)]
mod tests {

}
*/
