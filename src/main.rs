use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug, Clone)]
enum LispExpression {
    Number(f64),
    Boolean(bool),
    Symbol(String),
    List(Vec<LispExpression>),
    Lambda(Vec<String>, Box<LispExpression>),
}

#[derive(Debug, Clone)]
enum LispValue {
    Number(f64),
    Boolean(bool),
    Lambda(Vec<String>, Box<LispExpression>, Environment),
}

#[derive(Debug, Clone)]
struct Environment {
    bindings: HashMap<String, LispValue>,
}

impl Environment {
    fn new() -> Self {
        Environment {
            bindings: HashMap::new(),
        }
    }

    fn extend(&mut self, bindings: Vec<(String, LispValue)>) {
        self.bindings.extend(bindings.into_iter());
    }

    fn get(&self, key: &str) -> Option<&LispValue> {
        self.bindings.get(key)
    }

    fn set(&mut self, key: String, value: LispValue) {
        self.bindings.insert(key, value);
    }
}

fn eval(expr: &LispExpression, env: &mut Environment) -> Result<LispValue, String> {
    match expr {
        LispExpression::Number(num) => Ok(LispValue::Number(*num)),
        LispExpression::Boolean(b) => Ok(LispValue::Boolean(*b)),
        LispExpression::Symbol(sym) => {
            match env.get(sym) {
                Some(value) => Ok(value.clone()),
                None => Err(format!("Undefined symbol: {}", sym)),
            }
        }
        LispExpression::List(list) => {
            if list.is_empty() {
                return Err("Empty list".to_string());
            }
            match &list[0] {
                LispExpression::Symbol(s) if s == "define" => {
                    if list.len() != 3 {
                        return Err("Invalid define expression".to_string());
                    }
                    if let LispExpression::Symbol(name) = &list[1] {
                        let value = eval(&list[2], env)?;
                        env.set(name.clone(), value.clone());
                        Ok(value)
                    } else {
                        Err("Invalid variable name in define".to_string())
                    }
                }
                LispExpression::Symbol(s) if s == "lambda" => {
                    if list.len() != 3 {
                        return Err("Invalid lambda expression".to_string());
                    }
                    if let LispExpression::List(params) = &list[1] {
                        let mut param_names = Vec::new();
                        for param in params {
                            if let LispExpression::Symbol(name) = param {
                                param_names.push(name.clone());
                            } else {
                                return Err("Invalid parameter name in lambda".to_string());
                            }
                        }
                        let body = Box::new(list[2].clone());
                        Ok(LispValue::Lambda(
                            param_names,
                            body,
                            env.clone(), // Capture the current environment
                        ))
                    } else {
                        Err("Invalid parameter list in lambda".to_string())
                    }
                }
                _ => Err("Invalid expression".to_string()),
            }
        }
        LispExpression::Lambda(_, _) => Err("Lambda cannot be evaluated directly".to_string()),
    }
}
fn apply(func: &LispValue, args: &[LispExpression], env: &mut Environment) -> Result<LispValue, String> {
    match func {
        LispValue::Lambda(params, body, closure) => {
            if args.len() != params.len() {
                return Err("Incorrect number of arguments".to_string());
            }
            let mut new_env = closure.clone();
            for (param, arg) in params.iter().zip(args) {
                if let LispExpression::Symbol(name) = *param {
                    let value = eval(arg, env)?;
                    new_env.set(name.clone(), value);
                    println!("O símbolo é: {}", name);
                } else {
                    match arg {
                        LispExpression::Number(num) => println!("O parâmetro é um número: {}", num),
                        LispExpression::Boolean(b) => println!("O parâmetro é um booleano: {}", b),
                        LispExpression::List(_) => println!("O parâmetro é uma lista"),
                        LispExpression::Lambda(_, _) => println!("O parâmetro é uma lambda"),
                        _ => println!("Tipo de parâmetro desconhecido"),
                    }
                }
            }
            eval(&body, &mut new_env)
        }
        _ => Err("Invalid function application".to_string()),
    }
}

fn parse(tokens: &[&str]) -> Result<LispExpression, String> {
    let mut tokens = tokens.iter();
    parse_tokens(&mut tokens.map(|&s| s))
}

fn parse_tokens(tokens: &mut dyn Iterator<Item = &str>) -> Result<LispExpression, String> {
    let token = match tokens.next() {
        Some(token) => token,
        None => return Err("Unexpected end of input".to_string()),
    };

    match token {
        "(" => parse_list(tokens),
        ")" => Err("Unexpected ')'".to_string()),
        "true" => Ok(LispExpression::Boolean(true)),
        "false" => Ok(LispExpression::Boolean(false)),
        _ => {
            if let Ok(num) = token.parse::<f64>() {
                Ok(LispExpression::Number(num))
            } else {
                Ok(LispExpression::Symbol(token.to_string()))
            }
        }
    }
}

fn parse_list(tokens: &mut dyn Iterator<Item = &str>) -> Result<LispExpression, String> {
    let mut list = Vec::new();

    loop {
        let token = match tokens.next() {
            Some(token) => token,
            None => return Err("Unexpected end of input inside list".to_string()),
        };

        match token {
            "(" => {
                let sub_expr = parse_list(tokens)?;
                list.push(sub_expr);
            }
            ")" => return Ok(LispExpression::List(list)),
            _ => {
                let expr = parse_tokens(&mut std::iter::once(token))?;
                list.push(expr);
            }
        }
    }
}
fn main() {
    let mut env = Environment::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let trimmed_input = input.trim();

        if trimmed_input == "exit" {
            break;
        }

        let tokens: Vec<&str> = trimmed_input.split_whitespace().collect();
        match parse(&tokens) {
            Ok(expr) => match eval(&expr, &mut env) {
                Ok(value) => println!("{:?}", value),
                Err(err) => eprintln!("Error: {}", err),
            },
            Err(err) => eprintln!("Error parsing input: {}", err),
        }
    }
}
