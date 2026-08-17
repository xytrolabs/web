// Zor Formatter — pretty-prints Zor source code
use crate::ast::*;

pub fn format(stmts: &[Stmt]) -> String {
    let mut out = String::new();
    for (i, s) in stmts.iter().enumerate() {
        if i > 0 { out.push('\n'); }
        fmt_stmt(s, &mut out, 0);
    }
    out.push('\n');
    out
}

fn indent(n: usize) -> String { "    ".repeat(n) }

fn fmt_stmt(s: &Stmt, out: &mut String, depth: usize) {
    let ind = indent(depth);
    match s {
        Stmt::Fun(name, _, params, ret, body) => {
            out.push_str(&format!("{}fun {}(", ind, name));
            for (i, p) in params.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&format!("{} ", p.name));
                fmt_ty(&p.ty, out);
            }
            out.push(')');
            if let Some(r) = ret { out.push(' '); fmt_ty(r, out); }
            if body.len() == 1 && matches!(&body[0], Stmt::Give(Some(_))) {
                out.push_str(" = ");
                if let Stmt::Give(Some(e)) = &body[0] { fmt_expr(e, out); }
                out.push('\n');
            } else {
                out.push_str(" {\n");
                for b in body { fmt_stmt(b, out, depth + 1); }
                out.push_str(&format!("{}}}\n", ind));
            }
        },
        Stmt::Var(name, ty, val) => {
            out.push_str(&format!("{}var {} ", ind, name));
            if !matches!(ty, Type::Void) { fmt_ty(ty, out); out.push(' '); }
            if let Some(v) = val {
                out.push_str("= ");
                fmt_expr(v, out);
            }
            out.push('\n');
        },
        Stmt::Assign(name, val) => {
            out.push_str(&format!("{}{} = ", ind, name));
            fmt_expr(val, out);
            out.push('\n');
        },
        Stmt::Say(e) => {
            out.push_str(&format!("{}say: ", ind));
            fmt_expr(e, out);
            out.push('\n');
        },
        Stmt::Give(e) => {
            out.push_str(&format!("{}give: ", ind));
            if let Some(v) = e { fmt_expr(v, out); }
            out.push('\n');
        },
        Stmt::If(conds, else_body) => {
            for (i, (c, b)) in conds.iter().enumerate() {
                if i == 0 {
                    out.push_str(&format!("{}if ", ind));
                } else {
                    out.push_str(&format!("{}or if ", ind));
                }
                fmt_expr(c, out);
                out.push_str(" {\n");
                for s in b { fmt_stmt(s, out, depth + 1); }
                out.push_str(&format!("{}}} ", ind));
            }
            if let Some(eb) = else_body {
                out.push_str("otherwise {\n");
                for s in eb { fmt_stmt(s, out, depth + 1); }
                out.push_str(&format!("{}}}\n", ind));
            } else {
                out.push('\n');
            }
        },
        Stmt::Repeat(kind, body) => {
            out.push_str(&format!("{}repeat", ind));
            match kind {
                RepKind::Forever => {},
                RepKind::While(c) => { out.push_str(" while "); fmt_expr(c, out); },
                RepKind::Until(c) => { out.push_str(" until "); fmt_expr(c, out); },
                RepKind::Count(c) => { out.push(' '); fmt_expr(c, out); },
                RepKind::For(v, e) => { out.push_str(&format!(" {} in ", v)); fmt_expr(e, out); },
                RepKind::Range(v, s, e) => {
                    out.push_str(&format!(" {} in ", v));
                    fmt_expr(s, out); out.push_str(".."); fmt_expr(e, out);
                },
            }
            out.push_str(" {\n");
            for s in body { fmt_stmt(s, out, depth + 1); }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::Attempt(_, _) | Stmt::Spawn(_) | Stmt::Background(_) | Stmt::Await(_) | Stmt::Defer(_) => {}
            Stmt::Stop => { out.push_str(&format!("{}stop\n", ind)); },
        Stmt::Attempt(_, _) | Stmt::Spawn(_) | Stmt::Background(_) | Stmt::Await(_) | Stmt::Defer(_) => {}
            Stmt::Next => { out.push_str(&format!("{}next\n", ind)); },
        Stmt::StructDef(name, _, fields) => {
            out.push_str(&format!("{}struct {} {{\n", ind, name));
            for f in fields {
                out.push_str(&format!("{}    {} ", ind, f.name));
                fmt_ty(&f.ty, out);
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::EnumDef(name, _, variants) => {
            out.push_str(&format!("{}kind {} {{\n", ind, name));
            for v in variants {
                out.push_str(&format!("{}    {}", ind, v.name));
                if !v.types.is_empty() {
                    out.push('(');
                    for (i, t) in v.types.iter().enumerate() {
                        if i > 0 { out.push_str(", "); }
                        fmt_ty(t, out);
                    }
                    out.push(')');
                }
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::TraitDef(name, methods) => {
            out.push_str(&format!("{}trait {} {{\n", ind, name));
            for m in methods {
                out.push_str(&format!("{}    fun {}(", ind, m.name));
                for (i, p) in m.params.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(&format!("{} ", p.name));
                    fmt_ty(&p.ty, out);
                }
                out.push(')');
                if let Some(r) = &m.ret { out.push(' '); fmt_ty(r, out); }
                out.push('\n');
            }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::ImplDef(trait_name, type_name, body) => {
            out.push_str(&format!("{}impl {} for {} {{\n", ind, trait_name, type_name));
            for s in body { fmt_stmt(s, out, depth + 1); }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::Get(name, spec) => {
            out.push_str(&format!("{}get \"{}\"", ind, name));
            if let Some(s) = spec { out.push_str(&format!(" from {}", s)); }
            out.push('\n');
        },
        Stmt::Extern(funcs) => {
            out.push_str(&format!("{}extern \"C\" {{\n", ind));
            for ef in funcs {
                out.push_str(&format!("{}  fun {}(", ind, ef.name));
                let params: Vec<String> = ef.params.iter().map(|p| format!("{}: ", p.name)).collect();
                for (i, p) in ef.params.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(&format!("{}: ", p.name));
                    fmt_ty(&p.ty, out);
                }
                out.push(')');
                if let Some(ref r) = ef.ret { out.push_str(" -> "); fmt_ty(r, out); }
                out.push_str(";\n");
            }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::Unsafe(body) => {
            out.push_str(&format!("{}unsafe {{\n", ind));
            for s in body { fmt_stmt(s, out, depth + 1); }
            out.push_str(&format!("{}}}\n", ind));
        },
        Stmt::Use(path) => {
            out.push_str(&format!("{}use \"{}\"\n", ind, path));
        },
        Stmt::ExprStmt(e) => {
            out.push_str(&ind);
            fmt_expr(e, out);
            out.push('\n');
        },
        Stmt::FieldAssign(obj, field, val) => {
            out.push_str(&ind);
            fmt_expr(obj, out);
            out.push_str(&format!(".{} = ", field));
            fmt_expr(val, out);
            out.push('\n');
        },
    }
}

fn fmt_ty(t: &Type, out: &mut String) {
    match t {
        Type::Int => out.push_str("int"),
        Type::Float => out.push_str("float"),
        Type::String => out.push_str("string"),
        Type::Bool => out.push_str("bool"),
        Type::Void => out.push_str("void"),
        Type::Char => out.push_str("char"),
        Type::StrP => out.push_str("Str"),
        Type::Ptr(inner) => { out.push('*'); fmt_ty(inner, out); },
        Type::Ref(inner, is_mut) => { out.push('&'); if *is_mut { out.push_str("mut "); } fmt_ty(inner, out); },
        Type::Array(inner, n) => { out.push_str(&format!("[{}]", n)); fmt_ty(inner, out); },
        Type::Named(s) => out.push_str(s),
        Type::Generic(s) => out.push_str(s),
        Type::Instantiated(s, args) => {
            out.push_str(s); out.push('[');
            for (i, a) in args.iter().enumerate() { if i > 0 { out.push_str(", "); } fmt_ty(a, out); }
            out.push(']');
        },
        Type::VecP(inner) => { out.push_str("vec["); fmt_ty(inner, out); out.push(']'); },
        Type::MapP(k, v) => { out.push_str("map["); fmt_ty(k, out); out.push(']'); fmt_ty(v, out); },
    }
}

fn fmt_expr(e: &Expr, out: &mut String) {
    match e {
        Expr::Int(n) => out.push_str(&format!("{}", n)),
        Expr::Float(f) => out.push_str(&format!("{}", f)),
        Expr::Str(s) => { out.push('"'); out.push_str(s); out.push('"'); },
        Expr::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Expr::Var(n) => out.push_str(n),
        Expr::Bin(l, op, r) => {
            fmt_expr(l, out);
            out.push_str(&format!(" {} ", fmt_op(op)));
            fmt_expr(r, out);
        },
        Expr::Una(op, e) => {
            out.push_str(fmt_unop(op));
            fmt_expr(e, out);
        },
        Expr::Call(name, args) => {
            out.push_str(name); out.push('(');
            for (i, a) in args.iter().enumerate() { if i > 0 { out.push_str(", "); } fmt_expr(a, out); }
            out.push(')');
        },
        Expr::MethodCall(obj, method, args) => {
            fmt_expr(obj, out);
            out.push_str(&format!(".{}(", method));
            for (i, a) in args.iter().enumerate() { if i > 0 { out.push_str(", "); } fmt_expr(a, out); }
            out.push(')');
        },
        Expr::Field(obj, field) => { fmt_expr(obj, out); out.push('.'); out.push_str(field); },
        Expr::Index(base, idx) => { fmt_expr(base, out); out.push('['); fmt_expr(idx, out); out.push(']'); },
        Expr::Slice(base, start, end) => { fmt_expr(base, out); out.push('['); fmt_expr(start, out); out.push_str(".."); fmt_expr(end, out); out.push(']'); },
        Expr::Interp(parts) => {
            out.push('"');
            for p in parts {
                match p { Expr::Str(s) => out.push_str(s), Expr::Var(n) => { out.push('%'); out.push_str(n); out.push('%'); }, _ => {}, }
            }
            out.push('"');
        },
        Expr::StructLit(name, fields) => {
            out.push_str(name); out.push('{');
            for (i, (f, v)) in fields.iter().enumerate() { if i > 0 { out.push_str(", "); } out.push_str(f); out.push_str(": "); fmt_expr(v, out); }
            out.push('}');
        },
        Expr::EnumLit(name, variant, args) => {
            out.push_str(name); out.push_str("::"); out.push_str(variant);
            if !args.is_empty() {
                out.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    fmt_expr(a, out);
                }
                out.push(')');
            }
        },
        Expr::Match(subject, arms) => {
            out.push_str("case "); fmt_expr(subject, out); out.push_str(" {\n");
            for arm in arms {
                fmt_pattern(&arm.pattern, out);
                out.push_str(" => ");
                if arm.body.len() == 1 {
                    if let Stmt::ExprStmt(e) = &arm.body[0] { fmt_expr(e, out); out.push('\n'); }
                    else { fmt_stmt(&arm.body[0], out, 1); }
                } else {
                    out.push_str("{\n");
                    for s in &arm.body { fmt_stmt(s, out, 1); }
                    out.push_str("}\n");
                }
            }
            out.push('}');
        },
        Expr::New(e) => { out.push_str("new "); fmt_expr(e, out); },
        Expr::Free(e) => { out.push_str("free "); fmt_expr(e, out); },
        Expr::Deref(e) => { out.push('*'); fmt_expr(e, out); },
        Expr::AddrOf(e) => { out.push('&'); fmt_expr(e, out); },
        Expr::AddrOfMut(e) => { out.push_str("&mut "); fmt_expr(e, out); },
        Expr::Sizeof(t) => { out.push_str("sizeof("); fmt_ty(t, out); out.push(')'); },
        Expr::Closure(params, _, body) => {
            out.push_str("\\(");
            for (i, p) in params.iter().enumerate() { if i > 0 { out.push_str(", "); } out.push_str(&format!("{} ", p.name)); fmt_ty(&p.ty, out); }
            out.push(')');
            if body.len() == 1 && matches!(&body[0], Stmt::Give(Some(_))) {
                out.push_str(" -> ");
                if let Stmt::Give(Some(e)) = &body[0] { fmt_expr(e, out); }
            } else {
                out.push_str(" {\n");
                for s in body { fmt_stmt(s, out, 1); }
                out.push('}');
            }
        },
        Expr::Try(e) => { fmt_expr(e, out); out.push('?'); },
        Expr::Path(segs) => { out.push_str(&segs.join("::")); },
    }
}

fn fmt_pattern(p: &MatchPattern, out: &mut String) {
    match p {
        MatchPattern::Var(n) => out.push_str(n),
        MatchPattern::EnumVariant(n, binds) => {
            out.push_str(n);
            if !binds.is_empty() {
                out.push('(');
                for (i, b) in binds.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(b);
                }
                out.push(')');
            }
        },
        MatchPattern::Wildcard => out.push('_'),
        MatchPattern::Literal(l) => match l {
            LitPattern::Int(n) => out.push_str(&format!("{}", n)),
            LitPattern::Str(s) => { out.push('"'); out.push_str(s); out.push('"'); },
            LitPattern::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        },
        MatchPattern::Guard(inner, cond) => { fmt_pattern(inner, out); out.push_str(" if "); fmt_expr(cond, out); },
    }
}

fn fmt_op(op: &Op) -> &str {
    match op { Op::Add=>"+", Op::Sub=>"-", Op::Mul=>"*", Op::Div=>"/", Op::Mod=>"%", Op::Eq=>"==", Op::Neq=>"!=", Op::Lt=>"<", Op::Gt=>">", Op::Le=>"<=", Op::Ge=>">=", Op::And=>"&&", Op::Or=>"||", Op::BitOr=>"|", Op::BitAnd=>"&" }
}

fn fmt_unop(op: &UnOp) -> &str {
    match op { UnOp::Neg=>"-", UnOp::Not=>"!" }
}
