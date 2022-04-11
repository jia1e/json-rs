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

#[derive(Debug)]
pub enum JsonError {
    UnexpectToken,
}

fn skip_whitespace(chars: &Vec<char>, pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
}

fn expect_str(chars: &Vec<char>, pos: &mut usize, str: &str) -> Result<(), JsonError> {
    skip_whitespace(chars, pos);
    if *pos + str.len() <= chars.len() {
        for (i, ch) in str.chars().enumerate() {
            if chars[*pos + i] != ch {
                return Err(JsonError::UnexpectToken);
            }
        }
        Ok(())
    } else {
        Err(JsonError::UnexpectToken)
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
            expect_str(chars, pos, ":").unwrap();
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
        expect_str(chars, pos, ".").unwrap();
        number_string.push_str("0.");
        found_decimal = true;
        *pos += 1;
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
            expect_str(chars, pos, "true").unwrap();
            *pos += 4;
            Ok(Type::Boolean(true))
        }
        'f' => {
            expect_str(chars, pos, "false").unwrap();
            *pos += 5;
            Ok(Type::Boolean(false))
        }
        'n' => {
            expect_str(chars, pos, "null").unwrap();
            *pos += 4;
            Ok(Type::Null)
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
    use crate::{parse, Type};
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
