use std::{collections::HashMap, f64::consts::PI, fs, path::Path};

#[derive(Debug)]
enum Token {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Caret,
    Modulo,
    FloorDivision,
}

#[rustfmt::skip]
fn tokenize(s: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut s_iter = s.chars().peekable();

    while let Some(&c) = s_iter.peek() {
        match c {
            c if c.is_whitespace() => {s_iter.next();}
            '+' => {tokens.push(Token::Plus);s_iter.next();}
            '-' => {tokens.push(Token::Minus);s_iter.next();}
            '*' => {tokens.push(Token::Star);s_iter.next();}
            '/' => {
                s_iter.next();
                if s_iter.peek() == Some(&'/') {
                    tokens.push(Token::FloorDivision);
                    s_iter.next();
                } else {
                    tokens.push(Token::Slash);
                }
            }
            '(' => {tokens.push(Token::LParen);s_iter.next();}
            ')' => {tokens.push(Token::RParen);s_iter.next();}
            '^' => {tokens.push(Token::Caret);s_iter.next();}
            '%' => {tokens.push(Token::Modulo);s_iter.next();}

            c if c.is_ascii_digit() => {
                let mut num_buff = String::new();
                num_buff.push(c);
                s_iter.next();

                while let Some(&c) = s_iter.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_buff.push(c);
                        s_iter.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Num(
                    num_buff.parse::<f64>().expect("Failed to parse number"),
                ));
            }

            c if c.is_ascii_alphanumeric() || c == '_' => {
                let mut ident_buff = String::new();
                ident_buff.push(c);
                s_iter.next();

                while let Some(&c) = s_iter.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident_buff.push(c);
                        s_iter.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident_buff));
            }

            _ => panic!("Unexpected character : {c:?}"),
        }
    }
    tokens
}

#[rustfmt::skip]
fn eval(tokens: &[Token], vars: &HashMap<String, f64>) -> f64 {
    let mut pos = 0;

    fn sum(pos: &mut usize, tokens: &[Token], vars: &HashMap<String, f64>) -> f64 {
        let mut res = 0.0;
        while let Some(t) = tokens.get(*pos) {
            if matches!(t, Token::RParen) { break; }
            res += term(pos, tokens, vars);
        }
        res
    }

    fn power(pos: &mut usize, tokens: &[Token], vars: &HashMap<String, f64>) -> f64 {
        let mut val = probe(pos, tokens, vars);
        while let Some(Token::Caret) = tokens.get(*pos) {
            *pos += 1;
            val = val.powf(probe(pos, tokens, vars));
        }
        val
    }
    
    fn probe(pos: &mut usize, tokens: &[Token], vars: &HashMap<String, f64>) -> f64 {
        match tokens.get(*pos) {
            Some(Token::Num(n)) => { *pos += 1; *n }
            Some(Token::Minus)  => { *pos += 1; -probe(pos, tokens, vars) }
            Some(Token::Plus)   => { *pos += 1; probe(pos, tokens, vars) }
            Some(Token::LParen) => {
                *pos += 1;
                let v = sum(pos, tokens, vars);
                match tokens.get(*pos) {
                    Some(Token::RParen) => *pos += 1,
                    _ => panic!("missing )"),
                }
                v
            }
            Some(Token::Ident(name)) => {
                *pos += 1;
                match tokens.get(*pos) {
                    Some(Token::LParen) => {
                        let arg = probe(pos, tokens, vars);
                        match name.as_str() {
                                "sqrt"  => arg.sqrt(),
                                "ceil"  => arg.ceil(),
                                "floor" => arg.floor(),
                                "round" => arg.round(),
                                "sin"   => arg.sin(),
                                "cos"   => arg.cos(),
                                "exp"   => arg.exp(),
                                _ => panic!("unknown function: {name}"),
                            }
                    }
                    _ => match vars.get(name) {
                        Some(v) => *v,
                        None => panic!("unknown variable: {name}"),
                    },
                }
            }
            Some(huh) => panic!("expected number, got: {huh:?}"),
            None => panic!("expected number, got end of input"),
        }
    }

    fn term(pos: &mut usize, tokens: &[Token], vars: &HashMap<String, f64>) -> f64 {
        let mut val = power(pos, tokens, vars);
        loop {
            match tokens.get(*pos) {
                Some(Token::Star)  => { *pos += 1; val *= power(pos, tokens, vars); }
                Some(Token::Slash) => { *pos += 1; val /= power(pos, tokens, vars); }
                Some(Token::Modulo) => { *pos += 1; val %= power(pos, tokens, vars); }
                Some(Token::FloorDivision) => { *pos += 1; val = (val / power(pos, tokens, vars)).floor(); }
                
                _ => break,
            }
        }
        val
    }

    let res = sum(&mut pos, tokens, vars);
    if pos < tokens.len() {
        panic!("unexpected )");
    }
        
    res
}

fn process_file(text: &str) -> String {
    let mut variables: HashMap<String, f64> = HashMap::new();
    variables.insert("pi".to_string(), PI);

    let mut new_text = String::new();

    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            new_text.push('\n');
        }
        new_text.push_str(&process_line(line, &mut variables));
    }

    if text.ends_with('\n') {
        new_text.push('\n');
    }

    new_text
}

fn process_line(line: &str, vars: &mut HashMap<String, f64>) -> String {
    if line.matches('=').count() > 1 || line.matches("|>").count() > 1 {
        return line.to_string();
    }

    let (front, names) = match line.split_once("|>") {
        Some((f, n)) => (f, Some(n)),
        None => (line, None),
    };
    let (expr, has_slot) = match front.split_once('=') {
        Some((e, _old)) => (e, true),
        None => (front, false),
    };
    if names.is_none() && !has_slot {
        return line.to_string();
    }

    let value = eval(&tokenize(expr), vars);

    if let Some(n) = names {
        for name in n.split(',') {
            vars.insert(name.trim().to_string(), value);
        }
    }

    if !has_slot {
        return line.to_string();
    }

    let mut out = format!("{expr}= {value:.4}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string();

    if let Some(n) = names {
        out.push_str(" |>");
        out.push_str(n);
    }
    out
}

fn main() {
    // println!("{:?}", tokenize("2 + 3.5 * sqrt(4)"));
    // println!("{:?}", tokenize("iron_plate * 2"));

    // println!("{:?}", tokenize("sqrt(16) + 1"));

    // let mut variables: HashMap<String, f64> = HashMap::new();
    // let result = eval(&tokenize("10 % 3 + ceil(0.1)"), &mut variables);

    // println!("Result: {:?}", result);

    let path = Path::new("test.lichba");
    if !path.is_file() {
        panic!("File not found!")
    }

    let contents = fs::read_to_string(path).expect("Should have been able to read the file");

    let processed = process_file(&contents);

    fs::write(path, processed).expect("Failed to write");
}
