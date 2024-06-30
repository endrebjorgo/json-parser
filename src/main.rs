use std::{env, fs};
use std::process::exit;

type Result<T> = std::result::Result<T, ()>;

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

fn parse_json(bytes: Vec<u8>) -> Vec<String> {
    // TODO: Make recursive? Would help...
    let mut tokens = tokenize(&bytes).into_iter();
    loop {
        match tokens.next() {
            Some(token) => {
                match token.as_str() {
                    "{" => {
                        // Parsing an object
                        let mut keys = Vec::new();
                        //let mut values = Vec::new();  Requires some thought...
                        loop {
                            if tokens.next().unwrap().as_str() == "}" {
                                break;
                            }
                            assert_eq!("\"", tokens.next().unwrap().as_str());
                            keys.push(tokens.next().unwrap());
                            assert_eq!(":", tokens.next().unwrap().as_str());
                            // TODO: Push value to values
                        }
                    },
                    "[" => {
                        // Parsing an array
                    },
                    /*
                    "[" => {
                        // Parsing an array
                        let mut array = Vec::new();
                        loop {
                            match tokens.next().unwrap().as_str() {
                                "]" => {
                                    break;
                                },
                                "\"" => {
                                    // Parsing string
                                    array.push(tokens.next().unwrap());
                                    assert_eq!("\"", tokens.next().unwrap().as_str());
                                },
                                _ => {
                                    // Parsing someting else
                                },
                            }
                            
                            match tokens.next().unwrap().as_str() {
                                "," => continue,
                                "]" => break,
                                _ => unreachable!(),
                            }
                        }
                    },
                    */
                    "\"" => {
                        let string = tokens.next().unwrap();
                        assert_eq!("\"", tokens.next().unwrap().as_str());
                    },
                    "true" => {
                        // Parsing true keyword (assuming outside of string)
                    },
                    "false" => {
                        // Parsing false keyword (assuming outside of string)
                    },
                    "null" => {
                        // Parsing null keyword (assuming outside of string)
                    },
                    _ => {
                        // Parsing something else...
                    }
                }
            },
            None => {
                break;
            },
        }
    }
    return tokens.collect(); // For now to match type
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
    let x = parse_json(bytes);

    Ok(())
}
