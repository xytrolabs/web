// Zor AST types

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fun, Var, Give, Say, If, Or, Otherwise, Repeat, While, In, Stop, Next, Until, Is,
    Attempt, Catch, Spawn, Background, Await, Defer, Extern, Unsafe, Mut, Async, Use, Return, Loop_,
    Struct, New, Free, Sizeof, Len, Null, True, False, Empty,
    Enum, Match, ColonColon, Get, From, Trait, Impl, Self_, For, Backslash, Vec, Map,
    IntLit(i64), FloatLit(f64), StringLit(String), Ident(String),
    Colon, Semicolon, LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Dot, Eq, Plus, Minus, Star, Slash, Percent, Ampersand,
    EqEq, NotEq, Lt, Gt, LtEq, GtEq, AndAnd, OrOr, Not, Arrow, ThinArrow, FatArrow, DotDot, Question, Pipe,
    Eof, Newline, Indent, Dedent,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int, Float, String, Bool, Void, Char,
    Ptr(Box<Type>),
    Ref(Box<Type>, bool),  // &T (false) or &mut T (true)
    Array(Box<Type>, usize),
    Named(String),
    Generic(String),
    Instantiated(String, Vec<Type>),
    VecP(Box<Type>),
    MapP(Box<Type>, Box<Type>),
    StrP,  // owned String type (ptr, len, cap)
}

#[derive(Debug, Clone)]
pub struct Param { pub name: String, pub ty: Type }

#[derive(Debug, Clone)]
pub struct StructField { pub name: String, pub ty: Type }

#[derive(Debug, Clone)]
pub struct EnumVariant { pub name: String, pub types: Vec<Type> }

#[derive(Debug, Clone)]
pub struct MatchArm { pub pattern: MatchPattern, pub body: Vec<Stmt> }

#[derive(Debug, Clone)]
pub enum MatchPattern {
    Var(String),
    EnumVariant(String, Vec<String>),
    Wildcard,
    Literal(LitPattern),
    Guard(Box<MatchPattern>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum LitPattern {
    Int(i64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64), Float(f64), Str(String), Bool(bool), Var(String),
    Bin(Box<Expr>, Op, Box<Expr>), Una(UnOp, Box<Expr>),
    Call(String, Vec<Expr>), Interp(Vec<Expr>),
    Field(Box<Expr>, String), Index(Box<Expr>, Box<Expr>),
    Slice(Box<Expr>, Box<Expr>, Box<Expr>),
    StructLit(String, Vec<(String, Expr)>),
    EnumLit(String, String, Vec<Expr>),
    Match(Box<Expr>, Vec<MatchArm>),
    New(Box<Expr>), Deref(Box<Expr>), AddrOf(Box<Expr>), AddrOfMut(Box<Expr>), Free(Box<Expr>),
    Sizeof(Type),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    Closure(Vec<Param>, Option<Type>, Vec<Stmt>),
    Try(Box<Expr>),  // ? operator — propagate error
    Path(Vec<String>),  // crate::module::function
}

#[derive(Debug, Clone)]
pub enum Op { Add, Sub, Mul, Div, Mod, Eq, Neq, Lt, Gt, Le, Ge, And, Or, BitOr, BitAnd }

#[derive(Debug, Clone)]
pub enum UnOp { Neg, Not }

#[derive(Debug, Clone)]
pub enum Stmt {
    Var(String, Type, Option<Expr>), Assign(String, Expr), Say(Expr),
    Give(Option<Expr>), If(Vec<(Expr, Vec<Stmt>)>, Option<Vec<Stmt>>),
    Repeat(RepKind, Vec<Stmt>), Stop, Next, ExprStmt(Expr),
    Fun(String, Vec<String>, Vec<Param>, Option<Type>, Vec<Stmt>),
    StructDef(String, Vec<String>, Vec<StructField>),
    EnumDef(String, Vec<String>, Vec<EnumVariant>),
    Get(String, Option<String>),
    FieldAssign(Box<Expr>, String, Expr),
    TraitDef(String, Vec<TraitMethod>),
    ImplDef(String, String, Vec<Stmt>),
    Attempt(Vec<Stmt>, Option<Vec<Stmt>>),
    Spawn(Expr), Background(Expr), Await(Expr),
    Defer(Vec<Stmt>),
    Extern(Vec<ExternFn>),
    Unsafe(Vec<Stmt>),
    Use(String),
}

#[derive(Debug, Clone)]
pub enum RepKind { Forever, While(Expr), For(String, Expr), Count(Expr), Until(Expr), Range(String, Box<Expr>, Box<Expr>) }

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ExternFn {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Option<Type>,
    pub abi: String,  // "C", "system", etc.
}
