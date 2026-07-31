use crate::ast::Token;

pub struct Lexer { src: Vec<char>, pos: usize, pub line: usize }

impl Lexer {
    pub fn new(s: &str) -> Self { Lexer { src: s.chars().collect(), pos: 0, line: 1 } }
    fn peek(&self) -> Option<char> { self.src.get(self.pos).copied() }
    fn next(&mut self) -> Option<char> { let c = self.peek(); self.pos += 1; c }

    fn skip_ws(&mut self) {
        loop {
            while let Some(c) = self.peek() { if !c.is_whitespace() || c == '\n' { break } self.next(); }
            if self.peek() == Some('/') && self.src.get(self.pos + 1) == Some(&'/') {
                while let Some(c) = self.peek() { if c == '\n' { break } self.next(); }
            } else { break }
        }
    }

    fn read_string(&mut self) -> String {
        let mut s = String::new();
        while let Some(c) = self.next() {
            if c == '"' { return s }
            if c == '\\' { if let Some(n) = self.next() { s.push(match n { 'n'=>'\n','t'=>'\t','r'=>'\r','\\'=>'\\','"'=>'"',_=>n }) } }
            else { s.push(c) }
        }
        s
    }

    fn read_num(&mut self, first: char) -> Token {
        let mut s = String::from(first); let mut fl = false;
        // Hex literal: 0x or 0X prefix
        if first == '0' && self.peek().map_or(false, |c| c == 'x' || c == 'X') {
            s.push(self.next().unwrap_or('0')); // consume x/X
            while let Some(c) = self.peek() {
                if c.is_ascii_hexdigit() { s.push(self.next().unwrap_or('0')); }
                else { break }
            }
            return Token::IntLit(i64::from_str_radix(&s[2..], 16).unwrap_or(0));
        }
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() { s.push(self.next().unwrap_or('0')) }
            else if c == '.' && !fl {
                if self.pos + 1 < self.src.len() && self.src[self.pos + 1].is_ascii_digit() {
                    fl = true; s.push(self.next().unwrap_or('.'));
                } else { break }
            }
            else { break }
        }
        if fl { Token::FloatLit(s.parse().unwrap_or(0.0)) } else { Token::IntLit(s.parse().unwrap_or(0)) }
    }

    fn read_ident(&mut self, first: char) -> Token {
        let mut s = String::from(first);
        while let Some(c) = self.peek() { if c.is_alphanumeric() || c == '_' { s.push(self.next().unwrap_or('_')); } else { break } }
        match s.as_str() {
            "fun"=>Token::Fun,"var"=>Token::Var,"give"=>Token::Give,"say"=>Token::Say,
            "if"=>Token::If,"or"=>Token::Or,"otherwise"=>Token::Otherwise,
            "repeat"=>Token::Repeat,"while"=>Token::While,"in"=>Token::In,
            "stop"=>Token::Stop,"next"=>Token::Next,"until"=>Token::Until,"struct"=>Token::Struct,
            "try"=>Token::Attempt,"attempt"=>Token::Attempt,"catch"=>Token::Catch,
            "spawn"=>Token::Spawn,"background"=>Token::Background,"await"=>Token::Await,"wait"=>Token::Await,
            "defer"=>Token::Defer,
            "extern"=>Token::Extern,
            "unsafe"=>Token::Unsafe,
            "mut"=>Token::Mut,
            "async"=>Token::Async,
            "use"=>Token::Use,
            "return"=>Token::Return,
            "loop"=>Token::Loop_,
            "kind"=>Token::Enum,"enum"=>Token::Enum,"case"=>Token::Match,"match"=>Token::Match,
            "new"=>Token::New,"free"=>Token::Free,"sizeof"=>Token::Sizeof,
            "len"=>Token::Len,"null"=>Token::Null,"is"=>Token::Is,
            "get"=>Token::Get,"from"=>Token::From,
            "trait"=>Token::Trait,"impl"=>Token::Impl,"self"=>Token::Self_,"Self"=>Token::Self_,"for"=>Token::For,
            "vec"=>Token::Vec,"map"=>Token::Map,"true"=>Token::True,"false"=>Token::False,"empty"=>Token::Empty,
            _=>Token::Ident(s)
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut ts = Vec::new();
        loop {
            self.skip_ws();
            let t = match self.next() {
                None => { ts.push(Token::Eof); break }
                Some('\n') => { self.line += 1; Token::Newline },
                Some(':') => if self.peek()==Some(':'){self.next();Token::ColonColon}else{Token::Colon}, Some(';') => Token::Semicolon,
                Some('(') => Token::LParen, Some(')') => Token::RParen,
                Some('{') => Token::LBrace, Some('}') => Token::RBrace,
                Some('[') => Token::LBracket, Some(']') => Token::RBracket,
                Some(',') => Token::Comma, Some('.') => if self.peek()==Some('.'){self.next();Token::DotDot}else{Token::Dot},
                Some('&') => if self.peek()==Some('&'){self.next();Token::AndAnd}else{Token::Ampersand},
                Some('+') => Token::Plus,
                Some('-') => if self.peek()==Some('>'){self.next();Token::ThinArrow}else{Token::Minus},
                Some('*') => Token::Star, Some('/') => Token::Slash, Some('%') => Token::Percent,
                Some('=') => if self.peek()==Some('='){self.next();Token::EqEq}else if self.peek()==Some('>'){self.next();Token::FatArrow}else{Token::Eq},
                Some('!') => if self.peek()==Some('='){self.next();Token::NotEq}else{Token::Not},
                Some('<') => if self.peek()==Some('='){self.next();Token::LtEq}else{Token::Lt},
                Some('>') => if self.peek()==Some('='){self.next();Token::GtEq}else{Token::Gt},
                Some('\\') => Token::Backslash,
                Some('?') => Token::Question,
                Some('|') => if self.peek()==Some('|'){self.next();Token::OrOr}else{Token::Pipe},
                Some('"') => { let s = self.read_string(); Token::StringLit(s) },
                Some(c) if c.is_ascii_digit() => self.read_num(c),
                Some(c) if c.is_alphabetic()||c=='_' => self.read_ident(c),
                _ => continue,
            };
            ts.push(t);
        }
        ts
    }
}
