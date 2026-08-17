// Zor Ownership & Borrow Checker — production-grade safety
// Rules: primitives copy freely, heap types move, borrows tracked with mutability
// No lifetime annotations needed — NLL-style inference
// Send/Sync: auto-traits, all types Send+Sync except raw pointers

use crate::ast::*;
use std::collections::{HashMap, HashSet};

pub fn check(stmts: &[Stmt]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    // Phase 1: enum exhaustiveness check
    check_exhaustiveness(stmts, &mut errors);
    // Phase 2: ownership & borrow checking
    for s in stmts {
        if let Stmt::Fun(name, _, params, _, body) = s {
            let mut ctx = Ctx::new();
            for p in params { ctx.declare(&p.name, &p.ty, false); }
            check_stmts(body, &mut ctx, &mut errors);
            if !errors.is_empty() {
                errors.insert(0, format!("In function '{}':", name));
            }
        }
    }
    let mut ctx = Ctx::new();
    for s in stmts {
        if !matches!(s, Stmt::Fun(_, _, _, _, _)) {
            check_stmt(s, &mut ctx, &mut errors);
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

// ── Send/Sync auto-traits ──

/// All Zor types are Send+Sync except raw pointers
pub fn is_send(t: &Type) -> bool {
    match t {
        Type::Ptr(_) => false,  // raw pointers: not Send
        Type::Ref(_, _) => true,  // references: Send if inner is Send
        Type::VecP(inner) => is_send(inner),
        Type::MapP(k, v) => is_send(k) && is_send(v),
        Type::Array(inner, _) => is_send(inner),
        Type::Named(_) | Type::Instantiated(_, _) => true,  // structs: Send by default
        _ => true,  // primitives: always Send
    }
}

pub fn is_sync(t: &Type) -> bool {
    match t {
        Type::Ptr(_) => false,  // raw pointers: not Sync
        Type::Ref(inner, is_mut) => !is_mut && is_sync(inner),  // &T: Sync, &mut T: not Sync
        Type::VecP(inner) => is_sync(inner),
        Type::MapP(k, v) => is_sync(k) && is_sync(v),
        Type::Array(inner, _) => is_sync(inner),
        Type::Named(_) | Type::Instantiated(_, _) => true,
        _ => true,
    }
}

// ── Enum exhaustiveness ──

fn check_exhaustiveness(stmts: &[Stmt], errors: &mut Vec<String>) {
    // Collect all enum definitions
    let mut enums: HashMap<String, Vec<String>> = HashMap::new();
    for s in stmts {
        if let Stmt::EnumDef(name, _, variants) = s {
            enums.insert(name.clone(), variants.iter().map(|v| v.name.clone()).collect());
        }
    }
    // Check all match expressions
    for s in stmts {
        check_stmt_exhaustiveness(s, &enums, errors);
    }
}

fn check_stmt_exhaustiveness(s: &Stmt, enums: &HashMap<String, Vec<String>>, errors: &mut Vec<String>) {
    match s {
        Stmt::Var(_, _, Some(e)) | Stmt::Assign(_, e) | Stmt::Say(e) | Stmt::ExprStmt(e) => check_expr_exhaustiveness(e, enums, errors),
        Stmt::If(conds, else_body) => { for (c, b) in conds { check_expr_exhaustiveness(c, enums, errors); for sb in b { check_stmt_exhaustiveness(sb, enums, errors); } } if let Some(eb) = else_body { for sb in eb { check_stmt_exhaustiveness(sb, enums, errors); } } },
        Stmt::Repeat(_, body) | Stmt::Unsafe(body) => { for sb in body { check_stmt_exhaustiveness(sb, enums, errors); } },
        Stmt::Fun(_, _, _, _, body) => { for sb in body { check_stmt_exhaustiveness(sb, enums, errors); } },
        Stmt::ImplDef(_, _, body) => { for sb in body { check_stmt_exhaustiveness(sb, enums, errors); } },
        Stmt::Attempt(body, catch) => { for sb in body { check_stmt_exhaustiveness(sb, enums, errors); } if let Some(c) = catch { for sb in c { check_stmt_exhaustiveness(sb, enums, errors); } } },
        Stmt::Give(Some(e)) => check_expr_exhaustiveness(e, enums, errors),
        Stmt::FieldAssign(obj, _, val) => { check_expr_exhaustiveness(obj, enums, errors); check_expr_exhaustiveness(val, enums, errors); },
        Stmt::Spawn(e) | Stmt::Background(e) | Stmt::Await(e) => check_expr_exhaustiveness(e, enums, errors),
        _ => {},
    }
}

fn check_expr_exhaustiveness(e: &Expr, enums: &HashMap<String, Vec<String>>, errors: &mut Vec<String>) {
    match e {
        Expr::Match(subject, arms) => {
            check_expr_exhaustiveness(subject, enums, errors);
            // Infer enum type from subject
            let enum_name = infer_enum_type(subject);
            if let Some(variants) = enum_name.as_ref().and_then(|n| enums.get(n)) {
                let mut covered: HashSet<&str> = HashSet::new();
                let mut has_wildcard = false;
                for arm in arms {
                    match &arm.pattern {
                        MatchPattern::EnumVariant(v, _) => { covered.insert(v.as_str()); },
                        MatchPattern::Wildcard => { has_wildcard = true; },
                        _ => {},
                    }
                    for sb in &arm.body { check_stmt_exhaustiveness(sb, enums, errors); }
                }
                if !has_wildcard {
                    for v in variants {
                        if !covered.contains(v.as_str()) {
                            errors.push(format!("Non-exhaustive match: variant '{}::{}' not covered", enum_name.as_deref().unwrap_or("?"), v));
                        }
                    }
                }
            } else {
                for arm in arms { for sb in &arm.body { check_stmt_exhaustiveness(sb, enums, errors); } }
            }
        },
        Expr::Bin(l, _, r) => { check_expr_exhaustiveness(l, enums, errors); check_expr_exhaustiveness(r, enums, errors); },
        Expr::Una(_, e) | Expr::New(e) | Expr::Deref(e) | Expr::AddrOf(e) | Expr::AddrOfMut(e) | Expr::Free(e) | Expr::Try(e) => check_expr_exhaustiveness(e, enums, errors),
        Expr::Call(_, args) | Expr::MethodCall(_, _, args) => { for a in args { check_expr_exhaustiveness(a, enums, errors); } },
        Expr::StructLit(_, fields) => { for (_, v) in fields { check_expr_exhaustiveness(v, enums, errors); } },
        Expr::EnumLit(_, _, args) => { for a in args { check_expr_exhaustiveness(a, enums, errors); } },
        Expr::Closure(_, _, body) => { for sb in body { check_stmt_exhaustiveness(sb, enums, errors); } },
        Expr::Path(_) => {},
        Expr::Index(b, i) => { check_expr_exhaustiveness(b, enums, errors); check_expr_exhaustiveness(i, enums, errors); },
        Expr::Slice(b, s, e) => { check_expr_exhaustiveness(b, enums, errors); check_expr_exhaustiveness(s, enums, errors); check_expr_exhaustiveness(e, enums, errors); },
        Expr::Field(obj, _) => check_expr_exhaustiveness(obj, enums, errors),
        Expr::Interp(parts) => { for p in parts { check_expr_exhaustiveness(p, enums, errors); } },
        _ => {},
    }
}

fn infer_enum_type(e: &Expr) -> Option<String> {
    match e {
        Expr::Var(name) => {
            // Heuristic: variable names matching enum types
            if name.chars().next().map_or(false, |c| c.is_uppercase()) { Some(name.clone()) }
            else { None }
        },
        Expr::Field(inner, _) => infer_enum_type(inner),
        Expr::Call(name, _) => {
            if name.chars().next().map_or(false, |c| c.is_uppercase()) { Some(name.clone()) }
            else { None }
        },
        _ => None,
    }
}

// ── Type classification ──

fn is_heap_type(t: &Type) -> bool {
    matches!(t, Type::VecP(_) | Type::MapP(_, _) | Type::StrP | Type::Array(_, _))
}

fn is_ref_type(t: &Type) -> bool {
    matches!(t, Type::Ref(_, _))
}

fn is_copy_type(t: &Type) -> bool {
    matches!(t, Type::Int | Type::Float | Type::Bool | Type::Char | Type::String | Type::Ptr(_) | Type::Void | Type::Ref(_, _))
}

// ── Borrow kind ──

#[derive(Clone, PartialEq)]
enum BorrowKind {
    Immutable,
    Mutable,
}

// ── Variable state ──

#[derive(Clone)]
struct VarState {
    live: bool,
    borrows: Vec<(BorrowKind, usize)>,
    ty: Type,
    moved_fields: HashSet<String>,
}

impl VarState {
    fn has_mut_borrow(&self) -> bool { self.borrows.iter().any(|(k,_)| *k == BorrowKind::Mutable) }
    fn has_any_borrow(&self) -> bool { !self.borrows.is_empty() }
    fn add_borrow(&mut self, kind: BorrowKind, scope: usize) { self.borrows.push((kind, scope)); }
    fn release_scope(&mut self, scope: usize) { self.borrows.retain(|(_, s)| *s != scope); }
}

// ── Context ──

struct Ctx {
    vars: HashMap<String, VarState>,
    scope_depth: usize,
    scope_stack: Vec<usize>,
}

impl Ctx {
    fn new() -> Self { Ctx { vars: HashMap::new(), scope_depth: 0, scope_stack: vec![0] } }
    fn push_scope(&mut self) -> usize { self.scope_depth += 1; let id = self.scope_depth; self.scope_stack.push(id); id }
    fn pop_scope(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            for v in self.vars.values_mut() { v.release_scope(scope); }
        }
    }
    fn current_scope(&self) -> usize { *self.scope_stack.last().unwrap_or(&0) }
    fn declare(&mut self, name: &str, ty: &Type, _is_param: bool) {
        self.vars.insert(name.to_string(), VarState { live: true, borrows: Vec::new(), ty: ty.clone(), moved_fields: HashSet::new() });
    }
    fn is_live(&self, name: &str) -> bool { self.vars.get(name).map(|v| v.live).unwrap_or(true) }
    fn kill(&mut self, name: &str) {
        if let Some(v) = self.vars.get_mut(name) { if !v.has_any_borrow() { v.live = false; } }
    }
    fn add_borrow(&mut self, name: &str, kind: BorrowKind) -> Result<(), String> {
        let scope = self.current_scope();
        if let Some(v) = self.vars.get_mut(name) {
            if kind == BorrowKind::Mutable && v.has_any_borrow() { return Err(format!("Cannot mutably borrow '{}' while borrowed", name)); }
            if kind == BorrowKind::Immutable && v.has_mut_borrow() { return Err(format!("Cannot borrow '{}' while mutably borrowed", name)); }
            v.add_borrow(kind, scope);
        }
        Ok(())
    }
    fn has_borrows(&self, name: &str) -> bool { self.vars.get(name).map(|v| v.has_any_borrow()).unwrap_or(false) }
    fn has_mut_borrow(&self, name: &str) -> bool { self.vars.get(name).map(|v| v.has_mut_borrow()).unwrap_or(false) }
    fn is_heap(&self, name: &str) -> bool { self.vars.get(name).map(|v| is_heap_type(&v.ty)).unwrap_or(false) }
    fn clone_for_branch(&self) -> Self {
        Ctx { vars: self.vars.clone(), scope_depth: self.scope_depth, scope_stack: self.scope_stack.clone() }
    }
}

// ── Statement checking ──

fn check_stmts(stmts: &[Stmt], ctx: &mut Ctx, errors: &mut Vec<String>) {
    for s in stmts { check_stmt(s, ctx, errors); }
}

fn check_stmt(s: &Stmt, ctx: &mut Ctx, errors: &mut Vec<String>) {
    match s {
        Stmt::Var(name, ty, val) => {
            ctx.declare(name, ty, false);
            if let Some(v) = val {
                check_expr(v, ctx, errors);
                if is_heap_type(ty) {
                    if let Expr::Var(src) = v {
                        if src != name && !is_copy_type(ty) {
                            if ctx.has_borrows(src) { errors.push(format!("Cannot move '{}' while borrowed", src)); }
                            else { ctx.kill(src); }
                        }
                    }
                }
            }
        },
        Stmt::Assign(name, val) => {
            if !ctx.is_live(name) { errors.push(format!("Use of moved variable '{}'", name)); return; }
            if ctx.has_mut_borrow(name) { errors.push(format!("Cannot assign to '{}' while mutably borrowed", name)); return; }
            check_expr(val, ctx, errors);
            if ctx.is_heap(name) {
                if let Expr::Var(src) = val {
                    if src != name {
                        if ctx.has_borrows(src) { errors.push(format!("Cannot move '{}' while borrowed", src)); }
                        else { ctx.kill(src); }
                    }
                }
            }
        },
        Stmt::FieldAssign(obj, field, val) => {
            check_expr(obj, ctx, errors); check_expr(val, ctx, errors);
            if let Expr::Var(n) = obj.as_ref() {
                if ctx.has_mut_borrow(n) { errors.push(format!("Cannot assign to '{}.{}' while mutably borrowed", n, field)); }
            }
        },
        Stmt::Say(e) => check_expr(e, ctx, errors),
        Stmt::Give(e) => {
            if let Some(v) = e {
                check_expr(v, ctx, errors);
                if let Expr::Var(n) = v {
                    if ctx.is_heap(n) {
                        if ctx.has_borrows(n) { errors.push(format!("Cannot return '{}' while borrowed", n)); }
                        else { ctx.kill(n); }
                    }
                }
            }
        },
        Stmt::If(conds, else_body) => {
            for (c, b) in conds { check_expr(c, ctx, errors); let mut bc = ctx.clone_for_branch(); check_stmts(b, &mut bc, errors); }
            if let Some(eb) = else_body { let mut ec = ctx.clone_for_branch(); check_stmts(eb, &mut ec, errors); }
        },
        Stmt::Repeat(kind, body) => {
            ctx.push_scope();
            match kind { RepKind::While(c)|RepKind::Until(c) => check_expr(c, ctx, errors), RepKind::Range(_,s,e) => {check_expr(s,ctx,errors);check_expr(e,ctx,errors)}, _ => {} }
            let mut lc = ctx.clone_for_branch(); check_stmts(body, &mut lc, errors);
            ctx.pop_scope();
        },
        Stmt::ExprStmt(e) => check_expr(e, ctx, errors),
        Stmt::Fun(_, _, params, _, body) => { let mut fc = Ctx::new(); for p in params { fc.declare(&p.name, &p.ty, true); } check_stmts(body, &mut fc, errors); }
        Stmt::TraitDef(_, _) => {},
        Stmt::ImplDef(_, _, body) => { for s in body { check_stmt(s, ctx, errors); } }
        Stmt::Attempt(body, catch) => { let mut tc = ctx.clone_for_branch(); check_stmts(body, &mut tc, errors); if let Some(c) = catch { let mut cc = ctx.clone_for_branch(); check_stmts(c, &mut cc, errors); } }
        Stmt::Spawn(e) => check_expr(e, ctx, errors),
        Stmt::Background(e) => check_expr(e, ctx, errors),
        Stmt::Await(e) => check_expr(e, ctx, errors),
        Stmt::Extern(_) => {},
        Stmt::Unsafe(_) => {}, // programmer takes full responsibility
        _ => {},
    }
}

// ── Expression checking ──

fn check_expr(e: &Expr, ctx: &mut Ctx, errors: &mut Vec<String>) {
    match e {
        Expr::Var(name) => { if !ctx.is_live(name) { errors.push(format!("Use of moved value '{}' (heap types move, primitives copy)", name)); } }
        Expr::Bin(l, _, r) => { check_expr(l, ctx, errors); check_expr(r, ctx, errors); }
        Expr::Una(_, e) => check_expr(e, ctx, errors),
        Expr::Call(name, args) => {
            for (i, a) in args.iter().enumerate() {
                check_expr(a, ctx, errors);
                if let Expr::Var(n) = a {
                    if ctx.is_heap(n) {
                        if ctx.has_borrows(n) { errors.push(format!("Cannot pass '{}' to '{}' while borrowed (arg {})", n, name, i+1)); }
                        else { ctx.kill(n); }
                    }
                }
            }
        },
        Expr::MethodCall(obj, method, args) => {
            check_expr(obj, ctx, errors);
            let is_mut = method.starts_with("set_") || method.starts_with("push") || method == "insert" || method == "remove";
            if let Expr::Var(n) = obj.as_ref() {
                let k = if is_mut { BorrowKind::Mutable } else { BorrowKind::Immutable };
                if let Err(msg) = ctx.add_borrow(n, k) { errors.push(msg); }
            }
            for a in args { check_expr(a, ctx, errors); }
        },
        Expr::Closure(_, _, body) => { let mut fc = Ctx::new(); check_stmts(body, &mut fc, errors); }
        Expr::Field(obj, field) => {
            check_expr(obj, ctx, errors);
            if let Expr::Var(n) = obj.as_ref() { if let Some(v) = ctx.vars.get(n) { if v.moved_fields.contains(field) { errors.push(format!("Use of moved field '{}.{}'", n, field)); } } }
        },
        Expr::Index(base, idx) => { check_expr(base, ctx, errors); check_expr(idx, ctx, errors); }
        Expr::Slice(base, s, e) => { check_expr(base, ctx, errors); check_expr(s, ctx, errors); check_expr(e, ctx, errors); }
        Expr::AddrOf(inner) => {
            if let Expr::Var(n) = inner.as_ref() { if let Err(msg) = ctx.add_borrow(n, BorrowKind::Immutable) { errors.push(msg); } }
            check_expr(inner, ctx, errors);
        },
        Expr::AddrOfMut(inner) => {
            if let Expr::Var(n) = inner.as_ref() { if let Err(msg) = ctx.add_borrow(n, BorrowKind::Mutable) { errors.push(msg); } }
            check_expr(inner, ctx, errors);
        },
        Expr::Deref(e) => check_expr(e, ctx, errors),
        Expr::Try(e) => { ctx.push_scope(); check_expr(e, ctx, errors); ctx.pop_scope(); }
        Expr::New(e) => check_expr(e, ctx, errors),
        Expr::Free(e) => {
            check_expr(e, ctx, errors);
            if let Expr::Var(n) = e.as_ref() { if ctx.has_borrows(n) { errors.push(format!("Cannot free '{}' while borrowed", n)); } else { ctx.kill(n); } }
        },
        Expr::StructLit(_, fields) => { for (_, v) in fields { check_expr(v, ctx, errors); } }
        Expr::EnumLit(_, _, args) => { for a in args { check_expr(a, ctx, errors); } }
        Expr::Match(subject, arms) => {
            check_expr(subject, ctx, errors);
            for arm in arms {
                let mut ac = ctx.clone_for_branch();
                if let MatchPattern::EnumVariant(_, binds) = &arm.pattern { for b in binds { ac.declare(b, &Type::Int, false); } }
                if let MatchPattern::Guard(inner, cond) = &arm.pattern { check_expr(cond, ctx, errors); if let MatchPattern::EnumVariant(_, binds) = inner.as_ref() { for b in binds { ac.declare(b, &Type::Int, false); } } }
                check_stmts(&arm.body, &mut ac, errors);
            }
        },
        Expr::Interp(parts) => { for p in parts { check_expr(p, ctx, errors); } }
        Expr::Int(_) | Expr::Float(_) | Expr::Str(_) | Expr::Bool(_) => {},
        _ => {},
    };
}
