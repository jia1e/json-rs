use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Type>),
    Object(HashMap<String, Type>),
}

#[derive(Debug, PartialEq)]
pub enum JsonError {
    UnexpectToken,
}

fn skip_whitespace(chars: &Vec<char>, pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn find_str(chars: &Vec<char>, pos: &mut usize, str: &str) -> bool {
    skip_whitespace(chars, pos);
    if *pos + str.len() <= chars.len() {
        for (i, ch) in str.chars().enumerate() {
            if chars[*pos + i] != ch {
                return false;
            }
        }
        true
    } else {
        false
    }
}

fn parse_object(chars: &Vec<char>, pos: &mut usize) -> Result<Type, JsonError> {
    let mut hash: HashMap<String, Type> = HashMap::new();
    while *pos < chars.len() {
        skip_whitespace(chars, pos);

        if chars[*pos] == '}' {
            *pos += 1;
            return Ok(Type::Object(hash));
        }

        if let Type::String(key) = _parse(chars, pos).unwrap() {
            if find_str(chars, pos, ":") {
                *pos += 1;
                let value = _parse(chars, pos).unwrap();
                hash.insert(key, value);
                skip_whitespace(chars, pos);
                match chars[*pos] {
                    ',' => *pos += 1,
                    '}' => continue,
                    _ => break,
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }
    Err(JsonError::UnexpectToken)
}

fn parse_string(chars: &Vec<char>, pos: &mut usize) -> Result<Type, JsonError> {
    let mut result = String::new();

    while *pos < chars.len() {
        match chars[*pos] {
            '"' => {
                *pos += 1;
                return Ok(Type::String(result));
            }
            '\\' => {
                match chars[*pos + 1] {
                    'n' => result.push('\n'),
                    _ => result.push(chars[*pos + 1]),
                };
                *pos += 2;
            }
            ch @ _ => {
                result.push(ch);
                *pos += 1;
            }
        }
    }
    Err(JsonError::UnexpectToken)
}

fn parse_array(chars: &Vec<char>, pos: &mut usize) -> Result<Type, JsonError> {
    let mut result = Vec::<Type>::new();
    while *pos < chars.len() {
        skip_whitespace(chars, pos);
        match chars[*pos] {
            ',' if result.len() > 0 => {
                *pos += 1;
            }
            ',' => break,
            ']' => {
                *pos += 1;
                return Ok(Type::Array(result));
            }
            _ => {
                let value = _parse(chars, pos).unwrap();
                result.push(value);
            }
        }
    }
    Err(JsonError::UnexpectToken)
}

fn parse_number(chars: &Vec<char>, pos: &mut usize) -> Result<Type, JsonError> {
    let mut number_string = String::new();
    let mut found_decimal = false;
    let mut found_exponent = false;

    if chars[*pos] == '-' {
        number_string.push('-');
        *pos += 1;
    }

    if chars[*pos] == '0' {
        *pos += 1;
        if find_str(chars, pos, ".") {
            number_string.push_str("0.");
            found_decimal = true;
            *pos += 1;
        } else {
            return Err(JsonError::UnexpectToken);
        }
    }

    while *pos < chars.len() {
        match chars[*pos] {
            ch @ '0'..='9' => {
                number_string.push(ch);
                *pos += 1;
            }
            ch @ '.' => {
                if found_decimal || found_exponent {
                    return Err(JsonError::UnexpectToken);
                }
                found_decimal = true;
                number_string.push(ch);
                *pos += 1;
            }
            ch @ ('e' | 'E') => {
                if found_exponent {
                    return Err(JsonError::UnexpectToken);
                }
                found_exponent = true;
                number_string.push(ch);
                *pos += 1;

                match chars[*pos] {
                    ch @ ('-' | '+') => {
                        number_string.push(ch);
                        *pos += 1;
                    }
                    _ => {}
                }
            }
            _ => {
                break;
            }
        }
    }

    Ok(Type::Number(number_string.parse().unwrap()))
}

fn _parse(chars: &Vec<char>, pos: &mut usize) -> Result<Type, JsonError> {
    skip_whitespace(chars, pos);
    match chars[*pos] {
        '{' => {
            *pos += 1;
            parse_object(&chars, pos)
        }
        '[' => {
            *pos += 1;
            parse_array(chars, pos)
        }
        '"' => {
            *pos += 1;
            parse_string(&chars, pos)
        }
        't' => {
            if find_str(chars, pos, "true") {
                *pos += 4;
                Ok(Type::Boolean(true))
            } else {
                Err(JsonError::UnexpectToken)
            }
        }
        'f' => {
            if find_str(chars, pos, "false") {
                *pos += 5;
                Ok(Type::Boolean(false))
            } else {
                Err(JsonError::UnexpectToken)
            }
        }
        'n' => {
            if find_str(chars, pos, "null") {
                *pos += 4;
                Ok(Type::Null)
            } else {
                Err(JsonError::UnexpectToken)
            }
        }
        '0'..='9' | '-' => parse_number(chars, pos),
        _ => Err(JsonError::UnexpectToken),
    }
}

pub fn parse(json: &str) -> Result<Type, JsonError> {
    let chars: Vec<char> = json.chars().into_iter().collect();
    let mut pos: usize = 0;
    let result = _parse(&chars, &mut pos);
    skip_whitespace(&chars, &mut pos);
    if pos == chars.len() {
        result
    } else {
        Err(JsonError::UnexpectToken)
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, JsonError, Type};
    use std::collections::HashMap;

    #[test]
    fn it_works() {
        assert_eq!(parse("null").unwrap(), Type::Null);
        assert_eq!(parse("true").unwrap(), Type::Boolean(true));
        assert_eq!(parse("false").unwrap(), Type::Boolean(false));
        assert_eq!(parse("1").unwrap(), Type::Number(1.0));
        assert_eq!(parse("-1").unwrap(), Type::Number(-1.0));
        assert_eq!(parse("-1.1").unwrap(), Type::Number(-1.1));
        assert_eq!(parse("1e3").unwrap(), Type::Number(1000.0));
        assert_eq!(parse("1e+3").unwrap(), Type::Number(1000.0));
        assert_eq!(parse("1e-3").unwrap(), Type::Number(0.001));
        assert_eq!(parse("-1e-3").unwrap(), Type::Number(-0.001));
        assert_eq!(parse("01").unwrap_err(), JsonError::UnexpectToken);
        assert_eq!(parse("1.1.1").unwrap_err(), JsonError::UnexpectToken);
        assert_eq!(
            parse("\"hello world\"").unwrap(),
            Type::String("hello world".to_string())
        );
        assert_eq!(parse(" [ ] ").unwrap(), Type::Array(vec![]));
        assert_eq!(
            parse(" [ 1,-1 , null , true    , false, \"hello\", [ ] ] ").unwrap(),
            Type::Array(vec![
                Type::Number(1.0),
                Type::Number(-1.0),
                Type::Null,
                Type::Boolean(true),
                Type::Boolean(false),
                Type::String("hello".to_string()),
                Type::Array(vec![])
            ])
        );

        assert_eq!(parse("{ }").unwrap(), Type::Object(HashMap::new()));
        assert_eq!(
            parse("{ \"name\": \"json-rs\" }").unwrap(),
            Type::Object(HashMap::from_iter(vec![(
                "name".to_string(),
                Type::String("json-rs".to_string())
            )]))
        );
    }
}
