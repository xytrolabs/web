// Zor → Rust Transpiler
// Maps Zor AST directly to Rust source code.
// Supports #![no_std] for OS development via --target no_std.

use crate::ast::*;
use std::collections::HashMap;

pub struct RustGen {
    out: String,
    indent: usize,
    no_std: bool,
    // Track struct/enum names for method resolution
    structs: HashMap<String, Vec<(String, Type)>>,
    enums: HashMap<String, Vec<String>>,
    extern_names: std::collections::HashSet<String>,
}

impl RustGen {
    pub fn new(no_std: bool) -> Self {
        RustGen { out: String::new(), indent: 0, no_std,
            structs: HashMap::new(), enums: HashMap::new(),
            extern_names: std::collections::HashSet::new() }
    }

    fn w(&mut self, s: &str) {
        for _ in 0..self.indent { self.out.push_str("    "); }
        self.out.push_str(s);
        self.out.push('\n');
    }

    // Track Zor → Rust line mapping for error translation
    fn zor_line(&mut self, zor_ln: usize) {
        self.out.push_str(&format!("// ZOR:L{}\n", zor_ln));
    }

    fn ty(&self, t: &Type) -> String {
        match t {
            Type::Int => "i64".into(),
            Type::Float => "f64".into(),
            Type::String => if self.no_std { "&str".into() } else { "String".into() },
            Type::Bool => "bool".into(),
            Type::Char => "u8".into(),
            Type::Void => "()".into(),
            Type::Ptr(_) => "i64".into(),  // Zor pointers are i64 internally
            Type::Ref(inner, is_mut) => {
                if *is_mut { format!("&mut {}", self.ty(inner)) }
                else { format!("&{}", self.ty(inner)) }
            },
            Type::Array(inner, n) => format!("[{}; {}]", self.ty(inner), n),
            Type::VecP(inner) => format!("Vec<{}>", self.ty(inner)),
            Type::MapP(k, v) => format!("HashMap<{}, {}>", self.ty(k), self.ty(v)),
            Type::StrP => if self.no_std { "&str".into() } else { "String".into() },
            Type::Named(s) => s.clone(),
            Type::Generic(s) => s.clone(),
            Type::Instantiated(s, args) => {
                let args_str: Vec<String> = args.iter().map(|a| self.ty(a)).collect();
                format!("{}<{}>", s, args_str.join(", "))
            },
        }
    }

    fn expr(&self, e: &Expr) -> String {
        match e {
            Expr::Int(n) => n.to_string(),
            Expr::Float(f) => format!("{}", f),
            Expr::Str(s) => format!("r#\"{}\"#", s),
            Expr::Bool(true) => "true".into(),
            Expr::Bool(false) => "false".into(),
            Expr::Var(n) => n.clone(),
            Expr::Bin(l, op, r) => {
                let op_str = match op {
                    Op::Add => "+", Op::Sub => "-", Op::Mul => "*", Op::Div => "/",
                    Op::Mod => "%", Op::Eq => "==", Op::Neq => "!=",
                    Op::Lt => "<", Op::Gt => ">", Op::Le => "<=", Op::Ge => ">=",
                    Op::And => "&&", Op::Or => "||",
                    Op::BitOr => "|", Op::BitAnd => "&",
                };
                format!("({} {} {})", self.expr(l), op_str, self.expr(r))
            },
            Expr::Una(UnOp::Neg, e) => format!("(-{})", self.expr(e)),
            Expr::Una(UnOp::Not, e) => format!("(!{})", self.expr(e)),
            Expr::Call(name, args) => {
                let args_str: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                // Map runtime helpers to Rust equivalents
                match name.as_str() {
                    "zor_strcat" => format!("zor_strcat({})", args_str.join(", ")),
                    "zor_itoa" => format!("zor_itoa({})", args_str.join(", ")),
                    "zor_strlen" => format!("zor_strlen({})", args_str.join(", ")),
                    "zor_read_file" => format!("zor_read_file(&{})", args_str.join(", ")),
                    "zor_write_file" => {
                        if args_str.len() >= 2 {
                            format!("zor_write_file(&{}, &{})", args_str[0], args_str[1])
                        } else { format!("zor_write_file(&{})", args_str[0]) }
                    },
                    "zor_system" => format!("zor_system(&{})", args_str.join(", ")),
                    "zor_rc_alloc" => format!("zor_rc_alloc({})", args_str.join(", ")),
                    "zor_rc_release" => format!("zor_rc_release({})", args_str.join(", ")),
                    "system" => format!("zor_system(&{})", args_str.join(", ")),
                    "write_file" => {
                        if args_str.len() >= 2 {
                            format!("zor_write_file(&{}, &{})", args_str[0], args_str[1])
                        } else { format!("zor_write_file(&{})", args_str[0]) }
                    },
                    "read_file" => format!("zor_read_file(&{})", args_str.join(", ")),
                    "len" => {
                        if self.no_std {
                            format!("({}.len() as i64)", args_str.join(", "))
                        } else {
                            format!("({}.len() as i64)", args_str.join(", "))
                        }
                    },
                    _ => {
                        if self.extern_names.contains(name) {
                            format!("unsafe {{ {}({}) }}", name, args_str.join(", "))
                        } else {
                            format!("{}({})", name, args_str.join(", "))
                        }
                    },
                }
            },
            Expr::MethodCall(obj, method, args) => {
                let obj_str = self.expr(obj);
                let args_str: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                if method == "len" && args.is_empty() {
                    format!("({}.len() as i64)", obj_str)
                } else {
                    // Path::method or obj.method
                    format!("{}::{}({})", obj_str, method, args_str.join(", "))
                }
            },
            Expr::Field(obj, field) => format!("{}.{}", self.expr(obj), field),
            Expr::Index(obj, idx) => {
                // For &str indexing in no_std, use .as_bytes()[i]
                if self.no_std {
                    format!("({}).as_bytes()[{} as usize] as i64", self.expr(obj), self.expr(idx))
                } else {
                    format!("*(({}) as *mut i64).offset({} as isize)", self.expr(obj), self.expr(idx))
                }
            },
            Expr::Deref(e) => format!("*({} as *mut i64)", self.expr(e)),
            Expr::AddrOf(e) => format!("&{}", self.expr(e)),
            Expr::AddrOfMut(e) => format!("&mut {}", self.expr(e)),
            Expr::New(e) => format!("Box::new({})", self.expr(e)),
            Expr::Free(e) => format!("drop({})", self.expr(e)),
            Expr::StructLit(name, fields) => {
                let fields_str: Vec<String> = fields.iter().map(|(n, v)| format!("{}: {}", n, self.expr(v))).collect();
                format!("{} {{ {} }}", name, fields_str.join(", "))
            },
            Expr::EnumLit(ename, variant, args) => {
                let args_str: Vec<String> = args.iter().map(|a| self.expr(a)).collect();
                if args.is_empty() {
                    format!("{}::{}", ename, variant)
                } else {
                    format!("{}::{}({})", ename, variant, args_str.join(", "))
                }
            },
            Expr::Match(subject, arms) => {
                let mut s = format!("match {} {{\n", self.expr(subject));
                for arm in arms {
                    let pat = match &arm.pattern {
                        MatchPattern::Literal(LitPattern::Int(n)) => n.to_string(),
                        MatchPattern::Literal(LitPattern::Str(s)) => format!("\"{}\"", s),
                        MatchPattern::Literal(LitPattern::Bool(b)) => b.to_string(),
                        MatchPattern::Wildcard => "_".into(),
                        MatchPattern::Var(v) => v.clone(),
                        MatchPattern::EnumVariant(v, binds) => {
                            if binds.is_empty() { format!("{}::{}", "Enum", v) }
                            else { format!("{}::{}", "Enum", v) }
                        },
                        MatchPattern::Guard(_, _) => "_".into(),
                    };
                    s.push_str(&format!("    {} => {{\n", pat));
                    for st in &arm.body { s.push_str(&self.stmt_str(st, 2)); }
                    s.push_str("    },\n");
                }
                s.push('}');
                s
            },
            Expr::Try(e) => format!("({}?)", self.expr(e)),
            Expr::Sizeof(t) => format!("std::mem::size_of::<{}>() as i64", self.ty(t)),
            Expr::Interp(parts) => {
                let mut fmt = String::new();
                let mut args = vec![];
                for p in parts {
                    match p {
                        Expr::Str(s) => fmt.push_str(&s),
                        _ => { fmt.push_str("{}"); args.push(self.expr(p)); }
                    }
                }
                if args.is_empty() { format!("String::from(r#\"{}\"#)", fmt) }
                else { format!("format!(\"{}\", {})", fmt, args.join(", ")) }
            },
            Expr::Path(segments) => segments.join("::"),
            Expr::Closure(params, ret, body) => {
                let params_str: Vec<String> = params.iter().map(|p| p.name.clone()).collect();
                let mut body_str = String::new();
                for s in body { body_str.push_str(&self.stmt_str_inline(s)); }
                format!("|{}| {{ {} }}", params_str.join(", "), body_str.trim())
            },
            Expr::Slice(base, start, end) => format!("&{}[{} as usize..{} as usize]", self.expr(base), self.expr(start), self.expr(end)),
        }
    }

    fn stmt_str_inline(&self, s: &Stmt) -> String {
        match s {
            Stmt::Var(name, ty, Some(val)) => {
                format!("let mut {}: {} = {};\n", name, self.ty(ty), self.expr(val))
            },
            Stmt::Var(name, ty, None) => {
                format!("let mut {}: {} = Default::default();\n", name, self.ty(ty))
            },
            Stmt::Assign(name, val) => {
                format!("{} = {};\n", name, self.expr(val))
            },
            Stmt::Say(e) => format!("println!(\"{{}}\", {});\n", self.expr(e)),
            Stmt::Give(Some(e)) => format!("return {};\n", self.expr(e)),
            Stmt::Give(None) => "return;\n".into(),
            Stmt::ExprStmt(e) => format!("{};\n", self.expr(e)),
            Stmt::Stop => "break;\n".into(),
            Stmt::Next => "continue;\n".into(),
            Stmt::FieldAssign(obj, field, val) => {
                match obj.as_ref() {
                    Expr::Index(ptr, idx) => {
                        format!("*(({}) as *mut i64).offset({} as isize) = {};\n", self.expr(ptr), self.expr(idx), self.expr(val))
                    },
                    _ => format!("{}.{} = {}\n", self.expr(obj), field, self.expr(val)),
                }
            },
            _ => String::new(),
        }
    }

    fn stmt_str(&self, s: &Stmt, extra_indent: usize) -> String {
        let ind = "    ".repeat(self.indent + extra_indent);
        match s {
            Stmt::Var(name, ty, val) => {
                if let Some(v) = val {
                    format!("{}let mut {}: {} = {};\n", ind, name, self.ty(ty), self.expr(v))
                } else {
                    format!("{}let mut {}: {} = Default::default();\n", ind, name, self.ty(ty))
                }
            },
            Stmt::Assign(name, val) => format!("{}{} = {};\n", ind, name, self.expr(val)),
            Stmt::Say(e) => {
                if self.no_std {
                    format!("{}print({});\n", ind, self.expr(e))
                } else {
                    format!("{}println!(\"{{}}\", {});\n", ind, self.expr(e))
                }
            },
            Stmt::Give(Some(e)) => format!("{}return {};\n", ind, self.expr(e)),
            Stmt::Give(None) => format!("{}return;\n", ind),
            Stmt::ExprStmt(e) => format!("{}{};\n", ind, self.expr(e)),
            Stmt::Stop => format!("{}break;\n", ind),
            Stmt::Next => format!("{}continue;\n", ind),
            Stmt::FieldAssign(obj, field, val) => {
                match obj.as_ref() {
                    Expr::Index(ptr, idx) => {
                        format!("{}*(({}) as *mut i64).offset({} as isize) = {};\n", ind, self.expr(ptr), self.expr(idx), self.expr(val))
                    },
                    _ => format!("{}{}.{} = {};\n", ind, self.expr(obj), field, self.expr(val)),
                }
            },
            Stmt::If(conds, else_body) => {
                let mut s = String::new();
                for (i, (cond, body)) in conds.iter().enumerate() {
                    let kw = if i == 0 { "if" } else { "else if" };
                    s.push_str(&format!("{}{} {} {{\n", ind, kw, self.expr(cond)));
                    for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                    s.push_str(&format!("{}}}\n", ind));
                }
                if let Some(eb) = else_body {
                    s.push_str(&format!("{}else {{\n", ind));
                    for st in eb { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                    s.push_str(&format!("{}}}\n", ind));
                }
                s
            },
            Stmt::Repeat(kind, body) => {
                let mut s = String::new();
                match kind {
                    RepKind::Forever => {
                        s.push_str(&format!("{}loop {{\n", ind));
                        for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                        s.push_str(&format!("{}}}\n", ind));
                    },
                    RepKind::While(cond) => {
                        s.push_str(&format!("{}while {} {{\n", ind, self.expr(cond)));
                        for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                        s.push_str(&format!("{}}}\n", ind));
                    },
                    RepKind::Until(cond) => {
                        s.push_str(&format!("{}while !({}) {{\n", ind, self.expr(cond)));
                        for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                        s.push_str(&format!("{}}}\n", ind));
                    },
                    RepKind::Count(n) => {
                        s.push_str(&format!("{}for _ in 0..{} {{\n", ind, self.expr(n)));
                        for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                        s.push_str(&format!("{}}}\n", ind));
                    },
                    RepKind::For(var, expr) => {
                        s.push_str(&format!("{}for {} in {} {{\n", ind, var, self.expr(expr)));
                        for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                        s.push_str(&format!("{}}}\n", ind));
                    },
                    RepKind::Range(var, start, end) => {
                        s.push_str(&format!("{}for {} in {}..{} {{\n", ind, var, self.expr(start), self.expr(end)));
                        for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                        s.push_str(&format!("{}}}\n", ind));
                    },
                }
                s
            },
            Stmt::Fun(name, _, params, ret, body) => {
                let params_str: Vec<String> = params.iter().map(|p| format!("{}: {}", p.name, self.ty(&p.ty))).collect();
                let ret_str = match ret {
                    Some(t) => format!(" -> {}", self.ty(t)),
                    None => String::new(),
                };
                let mut s = format!("{}fn {}({}){} {{\n", ind, name, params_str.join(", "), ret_str);
                for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                s.push_str(&format!("{}}}\n", ind));
                s
            },
            Stmt::StructDef(name, _, fields) => {
                let fields_str: Vec<String> = fields.iter().map(|f| format!("{}pub {}: {}", ind, f.name, self.ty(&f.ty))).collect();
                format!("{}#[derive(Debug, Clone)]\n{}pub struct {} {{\n{}\n{}}}\n", ind, ind, name, fields_str.join(",\n"), ind)
            },
            Stmt::EnumDef(name, _, variants) => {
                let vars_str: Vec<String> = variants.iter().map(|v| {
                    if v.types.is_empty() {
                        format!("{}{}", ind, v.name)
                    } else {
                        let types: Vec<String> = v.types.iter().map(|t| self.ty(t)).collect();
                        format!("{}{}({})", ind, v.name, types.join(", "))
                    }
                }).collect();
                format!("{}#[derive(Debug, Clone)]\n{}pub enum {} {{\n{}\n{}}}\n", ind, ind, name, vars_str.join(",\n"), ind)
            },
            Stmt::ImplDef(trait_name, type_name, body) => {
                let mut s = format!("{}impl {} for {} {{\n", ind, trait_name, type_name);
                for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                s.push_str(&format!("{}}}\n", ind));
                s
            },
            Stmt::TraitDef(name, methods) => {
                let methods_str: Vec<String> = methods.iter().map(|m| {
                    let params: Vec<String> = m.params.iter().map(|p| format!("{}: {}", p.name, self.ty(&p.ty))).collect();
                    let ret = match &m.ret { Some(t) => format!(" -> {}", self.ty(t)), None => String::new() };
                    format!("{}fn {}({}){};\n", ind, m.name, params.join(", "), ret)
                }).collect();
                if methods.is_empty() {
                    format!("{}pub trait {} {{}}\n", ind, name)
                } else {
                    format!("{}pub trait {} {{\n{}{}}}\n", ind, name, methods_str.concat(), ind)
                }
            },
            Stmt::Extern(funcs) => {
                let funcs_str: Vec<String> = funcs.iter().map(|ef| {
                    let params: Vec<String> = ef.params.iter().map(|p| {
                        let ty = if matches!(p.ty, Type::String) { "i64".to_string() } else { self.ty(&p.ty) };
                        format!("{}: {}", p.name, ty)
                    }).collect();
                    let ret = match &ef.ret {
                        Some(t) if matches!(t, Type::String) => " -> i64".to_string(),
                        Some(t) => format!(" -> {}", self.ty(t)),
                        None => String::new(),
                    };
                    format!("{}pub fn {}({}){};", ind, ef.name, params.join(", "), ret)
                }).collect();
                format!("{}unsafe extern \"{}\" {{\n{}\n{}}}\n", ind, ef_funcs_first_abi(funcs), funcs_str.join("\n"), ind)
            },
            Stmt::Unsafe(body) => {
                let mut s = format!("{}unsafe {{\n", ind);
                for st in body {
                    // Wrap raw pointer operations inside unsafe block
                    s.push_str(&self.stmt_str(st, extra_indent + 1));
                }
                s.push_str(&format!("{}}}\n", ind));
                s
            },
            Stmt::Attempt(body, catch) => {
                // attempt { ... } catch { ... } → (|| { ... })().or_else(|_| { ... })
                let mut s = format!("{}(|| -> Result<(), Box<dyn std::error::Error>> {{\n", ind);
                for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                s.push_str(&format!("{}Ok(())\n", ind));
                s.push_str(&format!("{}}})().unwrap_or_else(|_| {{\n", ind));
                if let Some(cb) = catch { for st in cb { s.push_str(&self.stmt_str(st, extra_indent + 1)); } }
                s.push_str(&format!("{}}});\n", ind));
                s
            },
            Stmt::Spawn(e) => format!("{}tokio::spawn(async move {{ {}; }});\n", ind, self.expr(e)),
            Stmt::Background(e) => format!("{}tokio::spawn(async move {{ {}; }});\n", ind, self.expr(e)),
            Stmt::Await(e) => format!("{}{}.await;\n", ind, self.expr(e)),
            Stmt::Defer(body) => {
                let mut s = format!("{}// defer block (run at scope exit)\n", ind);
                s.push_str(&format!("{}let _zor_defer = scopeguard::guard((), |_| {{\n", ind));
                for st in body { s.push_str(&self.stmt_str(st, extra_indent + 1)); }
                s.push_str(&format!("{}}});\n", ind));
                s
            },
            Stmt::Use(path) => format!("{}use {};\n", ind, path),
            Stmt::Get(_, _) => String::new(), // Handled at module resolution level
        }
    }

    pub fn generate(&mut self, stmts: &[Stmt]) -> String {
        // Prelude
        if self.no_std {
            self.w("// Zor → Rust transpiler (no_std mode — for OS / embedded)");
            self.w("#![no_std]");
            self.w("#![no_main]");
            self.w("#![allow(unused_imports, unused_mut, unused_parens, unused_variables, dead_code, unused_unsafe)]");
            self.w("");

            // Minimal no_std runtime
            self.w("use core::panic::PanicInfo;");
            self.w("#[panic_handler]");
            self.w("fn panic(_info: &PanicInfo) -> ! { loop {} }");
            self.w("");
        } else {
            self.w("// Generated by Zor → Rust transpiler");
            self.w("#![allow(unused_imports, unused_mut, unused_parens, unused_variables, dead_code, unused_unsafe)]");
            self.w("use std::collections::HashMap;");
            self.w("");
        }

        // Runtime helpers (std mode only)
        self.emit_runtime();

        // Collect struct/enum definitions
        for s in stmts {
            match s {
                Stmt::StructDef(name, _, fields) => {
                    self.structs.insert(name.clone(), fields.iter().map(|f| (f.name.clone(), f.ty.clone())).collect());
                },
                Stmt::EnumDef(name, _, variants) => {
                    self.enums.insert(name.clone(), variants.iter().map(|v| v.name.clone()).collect());
                },
                _ => {},
            }
        }

        // Emit structs, enums, traits, externs
        for s in stmts {
            match s {
                Stmt::StructDef(_, _, _) | Stmt::EnumDef(_, _, _) | Stmt::TraitDef(_, _) | Stmt::Extern(_) => {
                    self.out.push_str(&self.stmt_str(s, 0));
                    self.out.push('\n');
                },
                _ => {},
            }
        }

        // no_std: emit global vars as static mut
        if self.no_std {
            for s in stmts {
                if let Stmt::Var(name, ty, val) = s {
                    let val_str = match val {
                        Some(v) => format!(" = {}", self.expr(v)),
                        None => String::new(),
                    };
                    self.w(&format!("static mut {}: {}{};", name, self.ty(ty), val_str));
                }
            }
            self.w("");
        }

        // Collect extern function names
        self.extern_names = stmts.iter()
            .filter_map(|s| if let Stmt::Extern(funcs) = s { Some(funcs.iter().map(|f| f.name.clone())) } else { None })
            .flatten()
            .collect();

        // Emit functions (skip extern-declared ones, rename main → zor_main)
        let has_main = stmts.iter().any(|s| matches!(s, Stmt::Fun(n, _, _, _, _) if n == "main"));
        let has_user_main = stmts.iter().any(|s| matches!(s, Stmt::Fun(n, _, _, _, _) if n == "main"));
        for s in stmts {
            match s {
                Stmt::Fun(name, _, params, ret, body) => {
                    if self.extern_names.contains(name) && !self.no_std { continue; }
                    let is_async = name.starts_with("__async__");
                    let real_name = if is_async { name[9..].to_string() } else { name.clone() };
                    let emit_name = if real_name == "main" { "zor_main".to_string() } else { real_name.clone() };
                    let fn_kw = if is_async { "async fn" } else { "fn" };
                    let params_str: Vec<String> = params.iter().map(|p| format!("{}: {}", p.name, self.ty(&p.ty))).collect();
                    let ret_str = match ret {
                        Some(t) => format!(" -> {}", self.ty(t)),
                        None => String::new(),
                    };
                    // no_std: add #[no_mangle] for kernel_main entry point
                    if self.no_std && emit_name == "kernel_main" {
                        self.w("#[no_mangle]");
                        self.w(&format!("pub extern \"C\" {} {}({}){} {{", fn_kw, emit_name, params_str.join(", "), ret_str));
                    } else {
                        self.w(&format!("{} {}({}){} {{", fn_kw, emit_name, params_str.join(", "), ret_str));
                    }
                    self.indent += 1;
                    for st in body {
                        self.out.push_str(&self.stmt_str(st, 1));
                    }
                    self.indent -= 1;
                    self.w("}");
                    self.w("");
                },
                Stmt::ImplDef(_, _, _) => {
                    self.out.push_str(&self.stmt_str(s, 0));
                    self.out.push('\n');
                },
                _ => {},
            }
        }

        // Main wrapper (only in std mode)
        if !self.no_std && has_user_main {
            let is_async_main = stmts.iter().any(|s| matches!(s, Stmt::Fun(n, _, _, _, _) if n == "__async__main"));
            if is_async_main {
                self.w("#[tokio::main]");
                self.w("async fn main() { zor_main().await; }");
            } else {
                self.w("fn main() { zor_main(); }");
            }
        } else if !self.no_std {
            // No user main — wrap top-level statements
            self.w("fn main() {");
            self.indent += 1;
            for s in stmts {
                if !matches!(s, Stmt::Fun(_,_,_,_,_)|Stmt::StructDef(_,_,_)|Stmt::EnumDef(_,_,_)|Stmt::TraitDef(_,_)|Stmt::ImplDef(_,_,_)|Stmt::Get(_,_)|Stmt::Extern(_)|Stmt::Use(_)) {
                    self.out.push_str(&self.stmt_str(s, 1));
                }
            }
            self.indent -= 1;
            self.w("}");
        }
        // no_std: no wrapper — user defines entry point

        self.out.clone()
    }

    fn emit_runtime(&mut self) {
        if self.no_std {
            return; // no_std uses inline stubs already emitted
        }
        self.w("// ─── Zor Runtime Helpers ───");
        self.w("");
        self.w("fn zor_strcat(a: &str, b: &str) -> String { format!(\"{}{}\", a, b) }");
        self.w("fn zor_itoa(n: i64) -> String { n.to_string() }");
        self.w("fn zor_strlen(s: &str) -> i64 { s.len() as i64 }");
        self.w("fn zor_read_file(path: &str) -> String { std::fs::read_to_string(path).unwrap_or_default() }");
        self.w("fn zor_write_file(path: &str, content: &str) { std::fs::write(path, content).ok(); }");
        self.w("fn zor_system(cmd: &str) -> i64 {");
        self.w("    std::process::Command::new(\"sh\").arg(\"-c\").arg(cmd).status().map(|s| s.code().unwrap_or(0) as i64).unwrap_or(-1)");
        self.w("}");
        self.w("fn zor_rc_alloc(size: i64) -> String {");
        self.w("    let mut v = vec![0u8; size as usize];");
        self.w("    String::from_utf8_lossy(&v).to_string()");
        self.w("}");
        self.w("fn zor_rc_release(_ptr: &str) {}");
        self.w("");
    }
}

fn ef_funcs_first_abi(funcs: &[ExternFn]) -> &str {
    funcs.first().map(|f| f.abi.as_str()).unwrap_or("C")
}
