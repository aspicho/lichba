
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

fn eval(tokens: &[Token]) -> f64 {
    let mut pos = 0;

    fn sum(pos: &mut usize, tokens: &[Token]) -> f64 {
        let mut res = 0.0;
        while let Some(t) = tokens.get(*pos) {
            if matches!(t, Token::RParen) { break; }
            res += term(pos, tokens);
        }
        res
    }

    fn power(pos: &mut usize, tokens: &[Token]) -> f64 {
        let mut val = probe(pos, tokens);
        while let Some(Token::Caret) = tokens.get(*pos) {
            *pos += 1;
            val = val.powf(probe(pos, tokens));
        }
        val
    }
    
    fn probe(pos: &mut usize, tokens: &[Token]) -> f64 {
        match tokens.get(*pos) {
            Some(Token::Num(n)) => { *pos += 1; *n }
            Some(Token::Minus)  => { *pos += 1; -probe(pos, tokens) }
            Some(Token::Plus)   => { *pos += 1; probe(pos, tokens) }
            Some(Token::LParen) => {
                *pos += 1;
                let v = sum(pos, tokens);
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
                        let arg = probe(pos, tokens);
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
                   _ => panic!("unknown variable: {name}"),
                }
            }
            Some(huh) => panic!("expected number, got: {huh:?}"),
            None => panic!("expected number, got end of input"),
        }
    }

    fn term(pos: &mut usize, tokens: &[Token]) -> f64 {
        let mut val = power(pos, tokens);
        loop {
            match tokens.get(*pos) {
                Some(Token::Star)  => { *pos += 1; val *= power(pos, tokens); }
                Some(Token::Slash) => { *pos += 1; val /= power(pos, tokens); }
                Some(Token::Modulo) => { *pos += 1; val %= power(pos, tokens); }
                Some(Token::FloorDivision) => { *pos += 1; val = (val / power(pos, tokens)).floor(); }
                
                _ => break,
            }
        }
        val
    }

    let res = sum(&mut pos, tokens);
    if pos < tokens.len() {
        panic!("unexpected )");
    }
        
    res
}

fn main() {
    println!("{:?}", tokenize("2 + 3.5 * sqrt(4)"));
    println!("{:?}", tokenize("iron_plate * 2"));

    println!("{:?}", tokenize("sqrt(16) + 1"));
    

    let result = eval(&tokenize("sqrt(16) + 1"));

    println!("Result: {:?}", result)
    
}
