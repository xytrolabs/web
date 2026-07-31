use crate::ast::*;
use crate::lexer::Lexer;

pub fn parse(source: &str) -> Vec<Stmt> { Parser::new(source).parse() }

struct Parser { tokens: Vec<Token>, pos: usize, line: usize, src: String }

impl Parser {
    fn new(src: &str) -> Self { let mut l=Lexer::new(src); let tokens=l.tokenize(); let line=l.line; Parser{tokens,pos:0,line, src: src.to_string()} }
    fn peek(&self) -> &Token { &self.tokens[self.pos] }
    fn next(&mut self) -> Token { let t=self.tokens[self.pos].clone();self.pos+=1;t }
    fn err(&self, msg: &str) -> ! { panic!("Zor error at line {}: {}", self.line, msg); }
    fn skipnl(&mut self) { while *self.peek()==Token::Newline {self.next();} }
    fn expect(&mut self, t: Token) { let tok=self.next(); if tok!=t { self.err(&format!("Expected {:?}, got {:?}",t,tok)); } }
    fn parse_name(&mut self) -> String {
        match self.next() {
            Token::Ident(x) => x,
            Token::Fun => "Fun".into(), Token::Var => "Var".into(), Token::Give => "Give".into(),
            Token::Say => "Say".into(), Token::If => "If".into(), Token::Or => "Or".into(),
            Token::Otherwise => "Otherwise".into(), Token::Repeat => "Repeat".into(),
            Token::While => "While".into(), Token::In => "In".into(), Token::Stop => "Stop".into(),
            Token::Next => "Next".into(), Token::Until => "Until".into(),
            Token::Struct => "Struct".into(), Token::New => "New".into(), Token::Free => "Free".into(),
            Token::Sizeof => "Sizeof".into(), Token::Len => "Len".into(), Token::True => "True".into(),
            Token::False => "False".into(), Token::Empty => "Empty".into(),
            Token::Enum => "kind".into(), Token::Match => "match".into(), Token::Get => "Get".into(),
            Token::From => "From".into(), Token::Trait => "Trait".into(), Token::Impl => "Impl".into(),
            Token::Self_ => "Self".into(), Token::For => "For".into(), Token::Vec => "Vec".into(),
            Token::Map => "Map".into(),
            t => panic!("Expected name, got {:?}", t),
        }
    }

    fn parse_ty(&mut self) -> Type {
        match self.next() {
            Token::Ident(s) => match s.as_str() {
                "int"=>Type::Int,"float"=>Type::Float,"string"=>Type::String,"bool"=>Type::Bool,"void"=>Type::Void,"char"=>Type::Char,
                "Str"=>Type::StrP,
                "vec"=>{
                    self.expect(Token::LBracket);
                    let inner = self.parse_ty();
                    self.expect(Token::RBracket);
                    Type::VecP(Box::new(inner))
                },
                _=> {
                    // Check if followed by [ -- instantiated generic like Box[int]
                    if *self.peek() == Token::LBracket {
                        self.next();
                        let mut args = vec![];
                        loop {
                            args.push(self.parse_ty());
                            if *self.peek() != Token::Comma { break }
                            self.next();
                        }
                        self.expect(Token::RBracket);
                        Type::Instantiated(s, args)
                    } else {
                        Type::Named(s)
                    }
                }
            },
            Token::Star => Type::Ptr(Box::new(self.parse_ty())),
            Token::Ampersand => {
                let is_mut = if *self.peek() == Token::Mut { self.next(); true } else { false };
                Type::Ref(Box::new(self.parse_ty()), is_mut)
            },
            Token::Vec => {
                self.expect(Token::LBracket);
                let inner = self.parse_ty();
                self.expect(Token::RBracket);
                Type::VecP(Box::new(inner))
            },
            Token::Map => {
                self.expect(Token::LBracket);
                let k = self.parse_ty();
                self.expect(Token::RBracket);
                let v = self.parse_ty();
                Type::MapP(Box::new(k), Box::new(v))
            },
            Token::LBracket => {
                if let Token::IntLit(n) = self.next() { self.expect(Token::RBracket); Type::Array(Box::new(self.parse_ty()), n as usize) }
                else { panic!("Expected array size") }
            },
            _ => panic!("Expected type"),
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.next() {
            Token::IntLit(n) => Expr::Int(n), Token::FloatLit(f) => Expr::Float(f),
            Token::StringLit(s) => self.parse_interp(s),
            Token::True => Expr::Bool(true), Token::False => Expr::Bool(false),
            Token::Null => Expr::Int(0),
            Token::Self_ => Expr::Var("self".into()),
            Token::Get => Expr::Var("get".into()),
            Token::Match => self.parse_match_expr(),
            Token::New => {
                let e = self.parse_primary();
                Expr::New(Box::new(e))
            },
            Token::Free => {
                Expr::Free(Box::new(self.parse_primary()))
            },
            Token::Star => Expr::Deref(Box::new(self.parse_primary())),
            Token::Ampersand => {
                // &expr or &mut expr
                let is_mut = if *self.peek() == Token::Mut { self.next(); true } else { false };
                let e = self.parse_primary();
                if is_mut { Expr::AddrOfMut(Box::new(e)) } else { Expr::AddrOf(Box::new(e)) }
            },
            Token::Sizeof => { self.expect(Token::LParen); let t = self.parse_ty(); self.expect(Token::RParen); Expr::Sizeof(t) },
            Token::Len => { self.expect(Token::LParen); let e = self.parse_expr(); self.expect(Token::RParen); Expr::Call("zor_strlen".into(), vec![e]) },
            Token::Ident(n) => {
                // Check for Enum::Variant
                if *self.peek()==Token::ColonColon || *self.peek()==Token::Dot {
                    // Path: gtk4::Application::new or gtk4.Application.new
                    let mut path = vec![n];
                    while *self.peek() == Token::ColonColon || *self.peek() == Token::Dot {
                        self.next(); // consume :: or .
                        let seg = match self.next() {
                            Token::Ident(s) => s,
                            Token::New => "new".into(),
                            t => panic!("Expected path segment, got {:?}", t),
                        };
                        path.push(seg);
                    }
                    if *self.peek() == Token::LParen {
                        self.next();
                        let mut a = vec![];
                        if *self.peek() != Token::RParen {
                            a.push(self.parse_expr());
                            while *self.peek() == Token::Comma { self.next(); a.push(self.parse_expr()); }
                        }
                        self.expect(Token::RParen);
                        let func = path.pop().unwrap();
                        return Expr::MethodCall(Box::new(Expr::Path(path)), func, a)
                    } else {
                        return Expr::Path(path)
                    }
                }
                if *self.peek()==Token::LParen { self.next(); let mut a=vec![];
                    if *self.peek()!=Token::RParen { a.push(self.parse_expr()); while *self.peek()==Token::Comma {self.next();a.push(self.parse_expr())} }
                    self.expect(Token::RParen); Expr::Call(n,a)
                } else if *self.peek()==Token::LBrace {
                    // Struct literal: Point { x: 10, y: 20 } or empty: Dog {}
                    // Only treat as struct literal if the ident starts with uppercase (type naming convention)
                    let is_type_name = n.chars().next().map_or(false, |c| c.is_uppercase());
                    if is_type_name {
                        self.next(); // consume {
                        self.skipnl();
                        if *self.peek() == Token::RBrace {
                            self.next();
                            Expr::StructLit(n, vec![])
                        } else {
                            let mut fields = vec![];
                            loop {
                                self.skipnl();
                                if *self.peek() == Token::RBrace { break }
                                let fname = self.parse_name();
                                self.expect(Token::Colon);
                                let fval = self.parse_expr();
                                fields.push((fname, fval));
                                self.skipnl();
                                if *self.peek() != Token::Comma { break }
                                self.next();
                            }
                            self.expect(Token::RBrace);
                            Expr::StructLit(n, fields)
                        }
                    } else {
                        Expr::Var(n)
                    }
                } else { Expr::Var(n) }
            },
            Token::LParen => { let e=self.parse_expr();self.expect(Token::RParen);e },
            Token::Minus => Expr::Una(UnOp::Neg,Box::new(self.parse_primary())),
            Token::Not => Expr::Una(UnOp::Not,Box::new(self.parse_primary())),
            Token::Backslash => {
                // Lambda: \(params) -> expr  or  \(params) { body }
                self.expect(Token::LParen);
                let mut ps = vec![];
                if *self.peek() != Token::RParen {
                    loop {
                        let pn = self.parse_name();
                        if *self.peek() == Token::Colon { self.next(); }
                        ps.push(Param { name: pn, ty: self.parse_ty() });
                        if *self.peek() != Token::Comma { break }
                        self.next();
                    }
                }
                self.expect(Token::RParen);
                // Optional -> arrow
                if *self.peek() == Token::ThinArrow {
                    self.next();
                }
                let body = if *self.peek() == Token::LBrace {
                    self.parse_block()
                } else {
                    vec![Stmt::Give(Some(self.parse_expr()))]
                };
                Expr::Closure(ps, None, body)
            },
            // Keywords usable as variable names
            Token::Fun => Expr::Var("Fun".into()), Token::Var => Expr::Var("Var".into()),
            Token::Give => Expr::Var("Give".into()), Token::Say => Expr::Var("Say".into()),
            Token::If => Expr::Var("If".into()), Token::Or => Expr::Var("Or".into()),
            Token::Otherwise => Expr::Var("Otherwise".into()), Token::Repeat => Expr::Var("Repeat".into()),
            Token::While => Expr::Var("While".into()), Token::In => Expr::Var("In".into()),
            Token::Stop => Expr::Var("Stop".into()), Token::Next => Expr::Var("Next".into()),
            Token::Until => Expr::Var("Until".into()), Token::Struct => Expr::Var("Struct".into()),
            Token::New => Expr::Var("New".into()), Token::Free => Expr::Var("Free".into()),
            Token::Sizeof => Expr::Var("Sizeof".into()), Token::Len => Expr::Var("Len".into()),
            Token::True => Expr::Bool(true), Token::False => Expr::Bool(false),
            Token::Empty => Expr::Var("Empty".into()), Token::Enum => Expr::Var("kind".into()),
            Token::Match => Expr::Var("match".into()), Token::Get => Expr::Var("Get".into()),
            Token::From => Expr::Var("From".into()), Token::Trait => Expr::Var("Trait".into()),
            Token::Impl => Expr::Var("Impl".into()), Token::Self_ => Expr::Var("Self".into()),
            Token::For => Expr::Var("For".into()), Token::Vec => Expr::Var("Vec".into()),
            Token::Map => Expr::Var("Map".into()), Token::Null => Expr::Int(0),
            Token::Is => Expr::Var("Is".into()),
            Token::Catch => Expr::Var("catch".into()), Token::Attempt => Expr::Var("attempt".into()),
            Token::Spawn => Expr::Var("spawn".into()), Token::Background => Expr::Var("background".into()),
            Token::Await => Expr::Var("await".into()),
            t => self.err(&format!("Unexpected token: {:?}",t)),
        }
    }

    fn parse_interp(&self, s: String) -> Expr {
        let mut p=vec![]; let mut c=String::new(); let ch:Vec<char>=s.chars().collect(); let mut i=0;
        while i<ch.len() { if ch[i]=='%' && i+1<ch.len() { if let Some(e)=ch[i+1..].iter().position(|&x|x=='%') { if !c.is_empty(){p.push(Expr::Str(c.clone()));c.clear()} p.push(Expr::Var(ch[i+1..i+1+e].iter().collect())); i+=e+2; continue; } } c.push(ch[i]); i+=1; }
        if p.is_empty() {Expr::Str(s)} else {if !c.is_empty(){p.push(Expr::Str(c))} Expr::Interp(p)}
    }

    fn parse_postfix(&mut self) -> Expr {
        let mut e = self.parse_primary();
        self.parse_postfix_from(e)
    }

    fn parse_postfix_from(&mut self, mut e: Expr) -> Expr {
        loop {
            match self.peek() {
                Token::LParen => {
                    // Function call: expr(args)
                    let name = match &e {
                        Expr::Var(n) => n.clone(),
                        _ => "unknown".into(),
                    };
                    self.next();
                    let mut args = vec![];
                    if *self.peek() != Token::RParen {
                        args.push(self.parse_expr());
                        while *self.peek() == Token::Comma { self.next(); args.push(self.parse_expr()); }
                    }
                    self.expect(Token::RParen);
                    e = Expr::Call(name, args);
                },
                Token::ColonColon => { self.next();
                    let variant = match self.next() { 
                        Token::Ident(v) => v, 
                        Token::Null => "None".into(),
                        _ => panic!("Expected variant name") 
                    };
                    let args = if *self.peek()==Token::LParen {
                        self.next();
                        let mut a = vec![];
                        if *self.peek() != Token::RParen {
                            a.push(self.parse_expr());
                            while *self.peek() == Token::Comma { self.next(); a.push(self.parse_expr()); }
                        }
                        self.expect(Token::RParen);
                        a
                    } else { vec![] };
                    // Extract the enum name from e
                    let ename = match &e {
                        Expr::Var(n) => n.clone(),
                        _ => "unknown".into(),
                    };
                    e = Expr::EnumLit(ename, variant, args);
                },
                Token::Dot => { self.next();
                    let f = self.parse_name();
                    // Check if this is a method call: x.method(args)
                    if *self.peek() == Token::LParen {
                        self.next();
                        let mut args = vec![];
                        if *self.peek() != Token::RParen {
                            args.push(self.parse_expr());
                            while *self.peek() == Token::Comma { self.next(); args.push(self.parse_expr()); }
                        }
                        self.expect(Token::RParen);
                        e = Expr::MethodCall(Box::new(e), f, args);
                    } else {
                        e = Expr::Field(Box::new(e), f);
                    }
                },
                Token::ThinArrow => { self.next(); // ->
                    let f = self.parse_name();
                    // Auto-deref: -> is just an alias for . (codegen handles pointer deref)
                    e = Expr::Field(Box::new(e), f);
                },
                Token::Question => {
                    self.next();
                    e = Expr::Try(Box::new(e));
                },
                Token::LBracket => { self.next();
                    let idx = self.parse_expr();
                    // Check for slice: v[1..3]
                    if *self.peek() == Token::DotDot {
                        self.next();
                        let end = self.parse_expr();
                        self.expect(Token::RBracket);
                        e = Expr::Slice(Box::new(e), Box::new(idx), Box::new(end));
                    } else {
                        self.expect(Token::RBracket);
                        e = Expr::Index(Box::new(e), Box::new(idx));
                    }
                },
                _ => break,
            }
        }
        e
    }

    fn parse_factor(&mut self) -> Expr { let mut e=self.parse_postfix(); loop { match self.peek() { Token::Star=>{self.next();e=Expr::Bin(Box::new(e),Op::Mul,Box::new(self.parse_postfix()))} Token::Slash=>{self.next();e=Expr::Bin(Box::new(e),Op::Div,Box::new(self.parse_postfix()))} Token::Percent=>{self.next();e=Expr::Bin(Box::new(e),Op::Mod,Box::new(self.parse_postfix()))} _=>break } } e }
    fn parse_term(&mut self) -> Expr { let mut e=self.parse_factor(); loop { match self.peek() { Token::Plus=>{self.next();e=Expr::Bin(Box::new(e),Op::Add,Box::new(self.parse_factor()))} Token::Minus=>{self.next();e=Expr::Bin(Box::new(e),Op::Sub,Box::new(self.parse_factor()))} Token::Pipe=>{self.next();e=Expr::Bin(Box::new(e),Op::BitOr,Box::new(self.parse_factor()))} _=>break } } e }
    fn parse_cmp(&mut self) -> Expr { let mut e=self.parse_term(); loop { match self.peek() { Token::EqEq=>{self.next();e=Expr::Bin(Box::new(e),Op::Eq,Box::new(self.parse_term()))} Token::NotEq=>{self.next();e=Expr::Bin(Box::new(e),Op::Neq,Box::new(self.parse_term()))} Token::Lt=>{self.next();e=Expr::Bin(Box::new(e),Op::Lt,Box::new(self.parse_term()))} Token::Gt=>{self.next();e=Expr::Bin(Box::new(e),Op::Gt,Box::new(self.parse_term()))} Token::LtEq=>{self.next();e=Expr::Bin(Box::new(e),Op::Le,Box::new(self.parse_term()))} Token::GtEq=>{self.next();e=Expr::Bin(Box::new(e),Op::Ge,Box::new(self.parse_term()))} _=>break } } e }
    fn parse_expr(&mut self) -> Expr { let mut e=self.parse_cmp(); loop { match self.peek() { Token::AndAnd=>{self.next();e=Expr::Bin(Box::new(e),Op::And,Box::new(self.parse_cmp()))} Token::OrOr=>{self.next();e=Expr::Bin(Box::new(e),Op::Or,Box::new(self.parse_cmp()))} Token::Or=>{self.next();e=Expr::Bin(Box::new(e),Op::Or,Box::new(self.parse_cmp()))} _=>break } } e }

    fn parse_block(&mut self) -> Vec<Stmt> {
        if *self.peek()==Token::LBrace { self.next(); let mut s=vec![];
            loop { self.skipnl(); if matches!(self.peek(),Token::RBrace|Token::Eof){break} s.push(self.parse_stmt()); }
            self.expect(Token::RBrace); s
        } else if *self.peek()==Token::Colon { self.next();
            if *self.peek()==Token::Newline { self.next(); let mut s=vec![];
                loop { self.skipnl(); if matches!(self.peek(),Token::Var|Token::Say|Token::Give|Token::If|Token::Repeat|Token::Stop|Token::Next|Token::Fun|Token::Struct|Token::Enum|Token::Ident(_)|Token::Star|Token::Return|Token::Loop_|Token::While){s.push(self.parse_stmt())}else{break} } s
            } else { vec![self.parse_stmt()] }
        } else { vec![self.parse_stmt()] }
    }

    fn parse_extern(&mut self) -> Stmt {
        // extern "C" { fn foo(x: int) -> int; fn bar() -> string; }
        self.next(); // consume 'extern'
        let abi = if let Token::StringLit(s) = self.next() { s } else { "C".into() };
        self.expect(Token::LBrace);
        let mut funcs = vec![];
        loop {
            self.skipnl();
            if *self.peek() == Token::RBrace { break }
            self.expect(Token::Fun);
            let name = match self.next() {
                Token::Ident(n) => n,
                t => panic!("Expected function name in extern block, got {:?}", t),
            };
            self.expect(Token::LParen);
            let mut params = vec![];
            if *self.peek() != Token::RParen {
                loop {
                    let pname = match self.next() { Token::Ident(n) => n, t => panic!("Expected param name, got {:?}", t), };
                    // Colon optional: `name type` or `name: type` both work
                    if *self.peek() == Token::Colon { self.next(); }
                    let pty = self.parse_ty();
                    params.push(Param { name: pname, ty: pty });
                    if *self.peek() != Token::Comma { break }
                    self.next();
                }
            }
            self.expect(Token::RParen);
            let ret = if *self.peek() == Token::ThinArrow { self.next(); Some(self.parse_ty()) } else { None };
            self.skipnl();
            funcs.push(ExternFn { name, params, ret, abi: abi.clone() });
        }
        self.expect(Token::RBrace);
        Stmt::Extern(funcs)
    }

    fn parse_match_expr(&mut self) -> Expr {
        // Match expression: match <expr> { arms }
        // Parse subject — use parse_primary which handles *, &, etc but NOT { for struct literals
        let subject = self.parse_primary();
        // If subject is a simple Var and next is {, it's the arms block (not struct literal)
        // parse_primary won't consume { because struct lit only triggers for uppercase names
        self.skipnl();
        self.expect(Token::LBrace);
        let mut arms = vec![];
        loop {
            self.skipnl();
            if *self.peek() == Token::RBrace { break }
            let pattern = match self.next() {
                Token::Ident(n) => {
                    if n == "_" { MatchPattern::Wildcard }
                    else if *self.peek() == Token::LParen {
                        self.next();
                        let mut binds = vec![];
                        loop {
                            if *self.peek() == Token::RParen { break }
                            let b = match self.next() { Token::Ident(b) => b, _ => panic!("Expected binding") };
                            binds.push(b);
                            if *self.peek() == Token::Comma { self.next(); }
                        }
                        self.expect(Token::RParen);
                        MatchPattern::EnumVariant(n, binds)
                    } else { MatchPattern::EnumVariant(n, vec![]) }
                },
                Token::Null => MatchPattern::EnumVariant("None".into(), vec![]),
                // Allow keyword tokens as variant names
                Token::Fun => MatchPattern::EnumVariant("Fun".into(), vec![]),
                Token::True => MatchPattern::EnumVariant("True".into(), vec![]),
                Token::False => MatchPattern::EnumVariant("False".into(), vec![]),
                Token::IntLit(n) => MatchPattern::Literal(LitPattern::Int(n)),
                Token::StringLit(s) => MatchPattern::Literal(LitPattern::Str(s)),
                _ => panic!("Expected pattern"),
            };
            // Check for guard: `if condition`
            let pattern = if *self.peek() == Token::If {
                self.next();
                let cond = self.parse_expr();
                MatchPattern::Guard(Box::new(pattern), Box::new(cond))
            } else { pattern };
            self.expect(Token::FatArrow);
            let body = if *self.peek()==Token::LBrace { self.parse_block() } else { vec![Stmt::ExprStmt(self.parse_expr())] };
            arms.push(MatchArm { pattern, body });
            self.skipnl();
        }
        self.expect(Token::RBrace);
        Expr::Match(Box::new(subject), arms)
    }

    fn parse_type_params(&mut self) -> Vec<String> {
        if *self.peek() == Token::LBracket {
            self.next();
            let mut params = vec![];
            loop {
                if let Token::Ident(n) = self.next() { params.push(n); }
                else { break }
                if *self.peek() != Token::Comma { break }
                self.next();
            }
            self.expect(Token::RBracket);
            params
        } else { vec![] }
    }

    fn parse_stmt(&mut self) -> Stmt {
        self.skipnl();
        match self.peek().clone() {
            Token::Var => { self.next(); let n=self.parse_name();
                // Accept `var x Type =` or `var x: Type =` (colon optional)
                if *self.peek() == Token::Colon { self.next(); }
                let t = if *self.peek() == Token::Eq {
                    // No type annotation — infer from value
                    Type::Void // Will be replaced during type inference
                } else {
                    self.parse_ty()
                };
                let v=if *self.peek()==Token::Eq {self.next();Some(self.parse_expr())}else{None}; Stmt::Var(n,t,v) },
            Token::Say => { self.next();
                if *self.peek()==Token::Colon {self.next(); Stmt::Say(self.parse_expr())}
                else if *self.peek()==Token::Semicolon { self.next();self.expect(Token::Newline); let mut ss=vec![];
                    loop { self.skipnl(); if let Token::StringLit(s)=self.peek().clone() { self.next(); ss.push(s); if *self.peek()==Token::Semicolon { self.next(); } else { break } } else { break } }
                    Stmt::Say(Expr::Str(ss.join("\n"))) } else { Stmt::Say(self.parse_expr()) } },
            Token::Return => { self.next(); Stmt::Give(Some(self.parse_expr())) },
            Token::Loop_ => { self.next(); let b=self.parse_block(); Stmt::Repeat(RepKind::Forever, b) },
            Token::Give => { self.next();
                // Colon is optional: `give expr` or `give: expr` both work
                if *self.peek() == Token::Colon { self.next(); }
                let val = if *self.peek() != Token::Newline && *self.peek() != Token::RBrace {
                    Some(self.parse_expr())
                } else { None };
                Stmt::Give(val)
            },
            Token::If => { self.next(); let c=self.parse_expr(); let b=self.parse_block(); let mut ei=vec![]; let mut el=None;
                loop { self.skipnl(); match self.peek() { Token::Or=>{self.next();self.skipnl();
                    if *self.peek()==Token::If {self.next();ei.push((self.parse_expr(),self.parse_block()))}else{el=Some(self.parse_block());break} }
                    Token::Otherwise=>{self.next();el=Some(self.parse_block());break} _=>break } }
                let mut cs=vec![(c,b)];cs.extend(ei); Stmt::If(cs,el) },
            Token::Repeat => { self.next();self.skipnl();
                let k=match self.peek() {
                    Token::While=>{self.next();RepKind::While(self.parse_expr())}
                    Token::Until=>{self.next();RepKind::Until(self.parse_expr())}
                    Token::Ident(_)=>{
                        let v=self.parse_name();
                        if *self.peek() == Token::In {
                            self.next();
                            let start = self.parse_expr();
                            if *self.peek() == Token::DotDot {
                                self.next();
                                let end = self.parse_expr();
                                RepKind::Range(v, Box::new(start), Box::new(end))
                            } else {
                                RepKind::For(v, start)
                            }
                        } else { panic!("Expected 'in' after loop variable") }
                    }
                    Token::LBrace=>RepKind::Forever,
                    _=>RepKind::Count(self.parse_expr())  // repeat N { ... }
                }; Stmt::Repeat(k,self.parse_block()) },
            Token::While => { self.next(); let c=self.parse_expr(); let b=self.parse_block(); Stmt::Repeat(RepKind::While(c), b) },
            Token::Stop=>{self.next();Stmt::Stop}, Token::Next=>{self.next();Stmt::Next},
            Token::Async => {
                self.next();
                self.expect(Token::Fun);
                let n = self.parse_name();
                let tps = self.parse_type_params();
                // Parentheses optional: `fun name` or `fun name()`
                let ps = if *self.peek() == Token::LParen {
                    self.next();
                    let mut params = vec![];
                    if *self.peek() != Token::RParen {
                        loop {
                            let pn = self.parse_name();
                            params.push(Param { name: pn, ty: self.parse_ty() });
                            if *self.peek() != Token::Comma { break }
                            self.next();
                        }
                    }
                    self.expect(Token::RParen);
                    params
                } else { vec![] };
                let r = if *self.peek() != Token::LBrace && *self.peek() != Token::Eq { Some(self.parse_ty()) } else { None };
                let body = if *self.peek() == Token::Eq { self.next(); vec![Stmt::Give(Some(self.parse_expr()))] } else { self.parse_block() };
                // Mark function as async — store as Fun with async flag
                // We reuse the existing Fun variant, async is handled by the transpiler
                Stmt::Fun(format!("__async__{}", n), tps, ps, r, body)
            },
            Token::Use => {
                self.next();
                let path = match self.next() {
                    Token::StringLit(s) => s,
                    Token::Ident(s) => s,
                    t => panic!("Expected string or identifier after use, got {:?}", t),
                };
                Stmt::Use(path)
            },
            Token::Fun => { self.next(); let n=self.parse_name(); let tps = self.parse_type_params();
                let ps = if *self.peek() == Token::LParen {
                    self.next();
                    let mut params = vec![];
                    if *self.peek() != Token::RParen {
                        loop {
                            let pn = self.parse_name();                            if *self.peek() == Token::Colon { self.next(); }                            if *self.peek() == Token::Colon { self.next(); }
                            params.push(Param { name: pn, ty: self.parse_ty() });
                            if *self.peek() != Token::Comma { break }
                            self.next();
                        }
                    }
                    self.expect(Token::RParen);
                    params
                } else { vec![] };
                let r = if *self.peek()!=Token::LBrace && *self.peek()!=Token::Eq { Some(self.parse_ty()) } else { None };
                let body=if *self.peek()==Token::Eq{self.next();vec![Stmt::Give(Some(self.parse_expr()))]}else{self.parse_block()};
                Stmt::Fun(n,tps,ps,r,body) },
            Token::Enum => { self.next();
                let n = match self.next() { Token::Ident(x) => x, _ => panic!("Expected enum name") };
                let tps = self.parse_type_params();
                self.expect(Token::LBrace);
                let mut variants = vec![];
                loop {
                    self.skipnl();
                    if *self.peek() == Token::RBrace { break }
                    let vname = match self.next() {
                        Token::Ident(x) => x,
                        Token::Null => "None".into(),
                        Token::Fun => "Fun".into(), Token::Var => "Var".into(), Token::Give => "Give".into(),
                        Token::Say => "Say".into(), Token::If => "If".into(), Token::Or => "Or".into(),
                        Token::Otherwise => "Otherwise".into(), Token::Repeat => "Repeat".into(),
                        Token::While => "While".into(), Token::In => "In".into(), Token::Stop => "Stop".into(),
                        Token::Next => "Next".into(), Token::Until => "Until".into(),
                        Token::Struct => "Struct".into(), Token::New => "New".into(), Token::Free => "Free".into(),
                        Token::Sizeof => "Sizeof".into(), Token::Len => "Len".into(), Token::True => "True".into(),
                        Token::False => "False".into(), Token::Empty => "Empty".into(),
                        Token::Enum => "kind".into(), Token::Match => "match".into(), Token::Get => "Get".into(),
                        Token::From => "From".into(), Token::Trait => "Trait".into(), Token::Impl => "Impl".into(),
                        Token::Self_ => "Self".into(), Token::For => "For".into(), Token::Vec => "Vec".into(),
                        Token::Map => "Map".into(), Token::Backslash => "Backslash".into(),
                        Token::Star => "Star".into(), Token::Ampersand => "Ampersand".into(),
                        t => panic!("Expected variant name, got {:?}", t),
                    };
                    let vtypes = if *self.peek() == Token::LParen {
                        self.next();
                        let mut types = vec![];
                        // Parse types until RParen (skip optional param names)
                        loop {
                            if *self.peek() == Token::RParen { break }
                            // Skip optional param name if present
                            let saved = self.pos;
                            if let Token::Ident(_) = self.next() {
                                if *self.peek() == Token::RParen || *self.peek() == Token::Comma {
                                    // It was just a name, no type → rewind and parse as type
                                    self.pos = saved;
                                } else {
                                    // Name followed by type → skip name
                                }
                            } else {
                                self.pos = saved;
                            }
                            types.push(self.parse_ty());
                            if *self.peek() == Token::Comma { self.next(); }
                        }
                        self.expect(Token::RParen);
                        types
                    } else { vec![] };
                    variants.push(EnumVariant { name: vname, types: vtypes });
                    self.skipnl();
                    if *self.peek() == Token::Comma { self.next(); self.skipnl(); }
                }
                self.expect(Token::RBrace);
                Stmt::EnumDef(n, tps, variants)
            },
            Token::Trait => { self.next();
                let n = match self.next() { Token::Ident(x) => x, _ => panic!("Expected trait name") };
                self.expect(Token::LBrace);
                let mut methods = vec![];
                loop {
                    self.skipnl();
                    if *self.peek() == Token::RBrace { break }
                    self.expect(Token::Fun);
                    let mn = match self.next() { Token::Ident(x) => x, _ => panic!("Expected method name") };
                    self.expect(Token::LParen);
                    let mut ps = vec![];
                    if *self.peek() != Token::RParen {
                        loop {
                            let pn = match self.next() { Token::Ident(x) => x, _ => panic!() };
                            ps.push(Param { name: pn, ty: self.parse_ty() });
                            if *self.peek() != Token::Comma { break }
                            self.next();
                        }
                    }
                    self.expect(Token::RParen);
                    let r = if *self.peek() == Token::ThinArrow { self.next(); Some(self.parse_ty()) }
                        else if *self.peek() != Token::Newline { Some(self.parse_ty()) } else { None };
                    methods.push(TraitMethod { name: mn, params: ps, ret: r });
                    self.skipnl();
                }
                self.expect(Token::RBrace);
                Stmt::TraitDef(n, methods)
            },
            Token::Impl => { self.next();
                let tn = match self.next() { Token::Ident(x) => x, _ => panic!("Expected trait name") };
                self.expect(Token::For);
                let tyn = match self.next() { Token::Ident(x) => x, _ => panic!("Expected type name") };
                self.expect(Token::LBrace);
                let mut body = vec![];
                loop {
                    self.skipnl();
                    if *self.peek() == Token::RBrace { break }
                    if *self.peek() == Token::Fun {
                        self.next();
                        let mn = match self.next() { Token::Ident(x) => x, _ => panic!("Expected method name") };
                        // Self parameter is implicit — skip if explicit
                        if *self.peek() == Token::LParen {
                            self.next();
                            // Skip self param
                            if *self.peek() == Token::Self_ { self.next(); }
                            else if *self.peek() != Token::RParen {
                                // Has explicit self param
                                let pn = match self.next() { Token::Ident(x) => x, _ => panic!() };
                                if pn == "self" { /* skip type after self */ if *self.peek() != Token::Comma && *self.peek() != Token::RParen { self.parse_ty(); } }
                            }
                            // Skip remaining params
                            while *self.peek() == Token::Comma {
                                self.next();
                                let _pn = match self.next() { Token::Ident(x) => x, _ => panic!() };
                                let _pt = self.parse_ty();
                            }
                            self.expect(Token::RParen);
                        }
                        let r = if *self.peek() == Token::ThinArrow { self.next(); Some(self.parse_ty()) }
                            else if *self.peek() != Token::LBrace && *self.peek() != Token::Eq {
                            Some(self.parse_ty())
                        } else { None };
                        let fn_body = if *self.peek() == Token::Eq {
                            self.next();
                            vec![Stmt::Give(Some(self.parse_expr()))]
                        } else { self.parse_block() };
                        // Desugar to Fun with Self replaced
                        let mut params = vec![Param { name: "self".into(), ty: Type::Named(tyn.clone()) }];
                        body.push(Stmt::Fun(format!("{}__{}", tyn, mn), vec![], params, r, fn_body));
                    } else { break }
                }
                self.expect(Token::RBrace);
                Stmt::ImplDef(tn, tyn, body)
            },
            Token::Struct => { self.next();
                let n = self.parse_name();
                let tps = self.parse_type_params();
                self.expect(Token::LBrace);
                let mut fields = vec![];
                loop {
                    self.skipnl();
                    if *self.peek() == Token::RBrace { break }
                    let fnm = self.parse_name();
                    let fty = self.parse_ty();
                    fields.push(StructField { name: fnm, ty: fty });
                    self.skipnl();
                    if *self.peek() == Token::Comma { self.next(); self.skipnl(); }
                }
                self.expect(Token::RBrace);
                Stmt::StructDef(n, tps, fields)
            },
            Token::Get => {
                self.next();
                if *self.peek() == Token::LParen {
                    // Function call: get(map, key)
                    let mut e = Expr::Var("get".into());
                    e = self.parse_postfix_from(e);
                    Stmt::ExprStmt(e)
                } else {
                    let n = match self.next() { Token::StringLit(s) => s, Token::Ident(s) => s, _ => panic!("Expected module name") };
                    let spec = if *self.peek() == Token::From { self.next(); Some(match self.next() { Token::Ident(x) => x, _ => panic!() }) } else { None };
                    Stmt::Get(n, spec)
                }
            },
            Token::Match => {
                self.next();
                Stmt::ExprStmt(self.parse_match_expr())
            },
            Token::Ident(name) => { self.next();
                if *self.peek()==Token::Eq { self.next(); Stmt::Assign(name,self.parse_expr()) }
                else if *self.peek()==Token::Is { self.next(); Stmt::Assign(name,self.parse_expr()) }
                else if *self.peek()==Token::Dot || *self.peek()==Token::ThinArrow {
                    let mut lhs = Expr::Var(name);
                    while *self.peek() == Token::Dot || *self.peek() == Token::ThinArrow {
                        self.next();
                        let fname = self.parse_name();
                        lhs = Expr::Field(Box::new(lhs), fname);
                    }
                    if *self.peek()==Token::Is { self.next(); }
                    else { self.expect(Token::Eq); }
                    Stmt::FieldAssign(Box::new(lhs), String::new(), self.parse_expr())
                }
                else if *self.peek()==Token::LBracket {
                    // Index assignment or expression: name[idx] = val
                    self.next();
                    let idx = self.parse_expr();
                    self.expect(Token::RBracket);
                    if *self.peek()==Token::Eq {
                        self.next();
                        Stmt::FieldAssign(Box::new(Expr::Index(Box::new(Expr::Var(name)), Box::new(idx))), String::new(), self.parse_expr())
                    } else {
                        Stmt::ExprStmt(self.parse_postfix_from(Expr::Index(Box::new(Expr::Var(name)), Box::new(idx))))
                    }
                }
                else {
                    // Function call: name(args) or name arg (zen mode — no parens)
                    let args = if *self.peek() == Token::LParen {
                        self.next();
                        let mut a = vec![];
                        if *self.peek() != Token::RParen {
                            a.push(self.parse_expr());
                            while *self.peek() == Token::Comma { self.next(); a.push(self.parse_expr()); }
                        }
                        self.expect(Token::RParen);
                        a
                    } else if matches!(self.peek(), Token::StringLit(_) | Token::IntLit(_) | Token::FloatLit(_) | Token::True | Token::False | Token::Null) {
                        let mut a = vec![self.parse_primary()];
                        while matches!(self.peek(), Token::StringLit(_) | Token::IntLit(_) | Token::FloatLit(_) | Token::True | Token::False | Token::Null) {
                            a.push(self.parse_primary());
                        }
                        a
                    } else { vec![] };
                    Stmt::ExprStmt(Expr::Call(name, args))
                }
            },
            Token::Star => {
                // Deref assignment: *ptr = value
                self.next();
                let lhs = self.parse_expr();
                self.expect(Token::Eq);
                Stmt::FieldAssign(Box::new(Expr::Deref(Box::new(lhs))), String::new(), self.parse_expr())
            },
            Token::Attempt => {
                self.next();
                let body = self.parse_block();
                let catch = if *self.peek() == Token::Catch { self.next(); Some(self.parse_block()) } else { None };
                Stmt::Attempt(body, catch)
            },
            Token::Spawn => {
                self.next();
                Stmt::Spawn(self.parse_expr())
            },
            Token::Background => {
                self.next();
                Stmt::Background(self.parse_expr())
            },
            Token::Await => {
                self.next();
                Stmt::Await(self.parse_expr())
            },
            Token::Defer => {
                self.next();
                Stmt::Defer(self.parse_block())
            },
            Token::Extern => self.parse_extern(),
            Token::Unsafe => {
                self.next();
                Stmt::Unsafe(self.parse_block())
            },
            // Expression statement: any expression-starting token
            Token::Free | Token::New | Token::Not | Token::Minus |
            Token::Backslash | Token::LParen | Token::IntLit(_) |
            Token::FloatLit(_) | Token::StringLit(_) | Token::True |
            Token::False | Token::Null | Token::Self_ |
            Token::Sizeof | Token::Len | Token::Ampersand |
            Token::Spawn | Token::Background | Token::Await | Token::Attempt => {
                Stmt::ExprStmt(self.parse_expr())
            },
            _ => self.err(&format!("Unexpected token: {:?}",self.peek())),
        }
    }

    fn parse(&mut self) -> Vec<Stmt> { let mut s=vec![]; loop{self.skipnl(); if *self.peek()==Token::Eof{break} let stmt = self.parse_stmt(); s.push(stmt);} s }
}
