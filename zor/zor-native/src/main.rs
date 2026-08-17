// Zor v0.5.0 — Xytro Labs
// ZOR = Zen-Oriented Rust. Clean syntax that transpiles to Rust.
// Write anything Rust can — OS kernels, apps, libraries — with less noise.

mod ast;
mod lexer;
mod parser;
mod codegen_rust;
mod ownership;
mod fmt;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::panic;

const PKG_CACHE: &str = ".zor/pkgs";

/// Parse Zor source, catching parser panics and converting to clean errors.
fn parse_safe(src: &str, filename: &str) -> Vec<ast::Stmt> {
    match panic::catch_unwind(panic::AssertUnwindSafe(|| parser::parse(src))) {
        Ok(stmts) => stmts,
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Syntax error — unexpected token or structure".to_string()
            };
            // Make parser panics look like proper errors
            let clean = msg.replace("Zor error at ", "")
                .replace(&format!("{}:", src.lines().count()), "end of file")
                .replace("panicked at", "error at");
            eprintln!("\n  ⚡ Parse error in {}: {}", filename, clean);
            eprintln!("     ── Check your syntax and try again.\n");
            std::process::exit(1);
        }
    }
}

fn pkg_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(PKG_CACHE)
}

fn compile_zor(input: &str, output: &str, _target: &str) {
    let src = std::fs::read_to_string(input).expect("Cannot read source");
    let mut stmts = parse_safe(&src, input);
    
    let mut resolved = Vec::new();
    let base = Path::new(input).parent().unwrap_or(Path::new("."));
    let pkg_dir = pkg_cache_dir();
    let search_paths: Vec<PathBuf> = vec![
        base.to_path_buf(),
        pkg_dir.clone(),
        base.join("std"),
        Path::new("std").to_path_buf(),
    ];
    
    for s in stmts {
        if let ast::Stmt::Get(module, _spec) = &s {
            let mut found = false;
            for sp in &search_paths {
                let pkg_dir = sp.join(module);
                if pkg_dir.is_dir() {
                    if let Some(latest) = find_latest_version(&pkg_dir) {
                        let mod_file = latest.join(format!("{}.zor", module));
                        if mod_file.exists() {
                            let mod_src = std::fs::read_to_string(&mod_file).unwrap_or_default();
                            let mod_stmts = parse_safe(&mod_src, &format!("{}.zor", module));
                            resolved.extend(mod_stmts);
                            found = true;
                            break;
                        }
                    }
                }
            }
            if !found {
                for sp in &search_paths {
                    let path = sp.join(format!("{}.zor", module));
                    if path.exists() {
                        let mod_src = std::fs::read_to_string(&path).unwrap_or_default();
                        let mod_stmts = parse_safe(&mod_src, &format!("{}.zor", module));
                        resolved.extend(mod_stmts);
                        found = true;
                        break;
                    }
                }
            }
            if !found {
                eprintln!("Zor: package '{}' not found", module);
            }
        } else {
            resolved.push(s);
        }
    }
    
    let resolved = monomorphize(resolved);
    
    if let Err(errs) = ownership::check(&resolved) {
        eprintln!("Zor: ownership errors:");
        for e in errs { eprintln!("  {}", e); }
        std::process::exit(1);
    }
    
    // Transpile to Rust, then compile with rustc
    let no_std = std::env::var("ZOR_NO_STD").is_ok();
    let mut gen = codegen_rust::RustGen::new(no_std);
    let rust_code = gen.generate(&resolved);
    
    let rs_file = format!("{}.rs", output);
    std::fs::write(&rs_file, &rust_code).expect("Cannot write Rust output");
    
    let mut rustc_args = vec!["-O", &rs_file, "-o", output];
    if no_std {
        rustc_args.push("-C");
        rustc_args.push("panic=abort");
        rustc_args.push("--emit=obj");
    }
    
    let result = Command::new("rustc")
        .args(&rustc_args)
        .output()
        .expect("rustc not found — install Rust: https://rustup.rs");
    
    if result.status.success() {
        std::fs::remove_file(&rs_file).ok();
        println!("Zor: -> ./{} (via Rust)", output);
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        eprintln!("{}", translate_rust_errors(&stderr, &src));
        eprintln!("Zor: Rust source saved to {}", rs_file);
        std::process::exit(1);
    }
}

/// Translate Rust compiler errors into Zor-friendly messages.
/// Uses // ZOR:L{line} comments in generated Rust for accurate line mapping,
/// falling back to a heuristic when not available.
fn translate_rust_errors(rust_err: &str, zor_src: &str) -> String {
    let zor_lines: Vec<&str> = zor_src.lines().collect();
    let mut result = String::new();
    let mut seen_explanations = std::collections::HashSet::new();
    let mut seen_context = std::collections::HashSet::new();
    
    // Build ZOR:L → line number map from the Rust error output
    // (the generated .rs file may still be on disk)
    let zor_map = build_zor_line_map(rust_err);
    
    for line in rust_err.lines() {
        // ── Translate known Rust error codes ──
        let explanation = if line.contains("error[E0308]") || line.contains("mismatched types") {
            if line.contains("expected `i64`, found `&str`") || line.contains("expected `String`, found `&str`") {
                "type mismatch → found a string, expected a number. Remove the quotes or convert."
            } else if line.contains("expected `&str`") || line.contains("expected `String`") {
                "type mismatch → found a number, expected a string. Add quotes: \"value\""
            } else if line.contains("expected `i64`, found `()`") {
                "missing return → function must return int. Add: return 0 at the end."
            } else if line.contains("expected `()`, found `i64`") {
                "unexpected return value → this function returns void. Remove the return value or add a return type."
            } else {
                "type mismatch → the types don't match. Check what you're passing and the function signature."
            }
        } else if line.contains("error[E0425]") {
            "undefined name → declare it with 'var' or import with 'use' / 'get'"
        } else if line.contains("error[E0432]") {
            "import not found → check the crate name in your 'use' statement. Did you add it to dependencies?"
        } else if line.contains("error[E0133]") {
            "FFI call needs unsafe → wrap in: unsafe { ... }"
        } else if line.contains("error[E0382]") {
            "value was moved → use &ref to borrow instead of moving ownership"
        } else if line.contains("error[E0502]") {
            "borrow conflict → can't borrow as mutable and immutable at the same time. Restructure your code."
        } else if line.contains("error[E0596]") {
            "cannot borrow as mutable → the variable isn't declared as mutable. Use: var x &mut T = ..."
        } else if line.contains("error[E0507]") {
            "cannot move out of borrowed content → use .clone() or restructure to avoid moving from a reference"
        } else if line.contains("error[E0317]") {
            "if without else → this if is used as an expression but has no else branch. Add 'else' or use a statement."
        } else if line.contains("error[E0369]") {
            "cannot combine types → convert with .to_string() or cast to matching types"
        } else if line.contains("error[E0601]") {
            "no main function → add: fun main int { ... }"
        } else if line.contains("error[E0603]") {
            "private item → the function or field is not public. Make it public or access it differently."
        } else if line.contains("error[E0599]") {
            "no method named → the type doesn't have this method. Check the struct/impl definition."
        } else if line.contains("error[E0277]") {
            "trait not satisfied → the type doesn't implement a required trait. Check trait bounds."
        } else if line.contains("error[E0061]") {
            "wrong number of arguments → check the function signature for the correct parameter count."
        } else if line.contains("error[E0412]") {
            "type not found → the type name is not in scope. Import it with 'use' or check spelling."
        } else if line.contains("error[E0433]") {
            "failed to resolve import → the module path may be wrong or the crate is missing."
        } else if line.contains("error[E0308]") || line.contains("mismatched types") {
            // Catch any remaining E0308 variants
            "type mismatch → the types don't match. Check what you're passing and the function signature."
        } else {
            ""
        };
        
        if !explanation.is_empty() && !seen_explanations.contains(explanation) {
            result.push_str(&format!("\n  ⚡ {}\n", explanation));
            seen_explanations.insert(explanation);
        }
        
        // Show Zor source context when Rust points to a line
        if line.contains("-->") {
            if let Some(rest) = line.split("-->").nth(1) {
                let parts: Vec<&str> = rest.trim().split(':').collect();
                if parts.len() >= 2 {
                    if let Ok(rs_ln) = parts[1].parse::<usize>() {
                        // Try accurate mapping first, fall back to heuristic
                        let zor_ln = zor_map.get(&rs_ln).copied()
                            .unwrap_or_else(|| (rs_ln as f64 * 0.7) as usize);
                        
                        if zor_ln > 0 && zor_ln <= zor_lines.len() && seen_context.len() < 5 {
                            let start = if zor_ln > 2 { zor_ln - 2 } else { 0 };
                            let end = (zor_ln + 3).min(zor_lines.len());
                            result.push_str(&format!("     ── near (Zor line ~{}):\n", zor_ln));
                            for i in start..end {
                                let marker = if i + 1 == zor_ln { "▶" } else { " " };
                                result.push_str(&format!("     {} {:4} │ {}\n", marker, i + 1, zor_lines[i]));
                            }
                            seen_context.insert(zor_ln);
                        }
                    }
                }
            }
        }
    }
    
    if result.is_empty() {
        "Zor: compilation failed. Check for syntax errors.\n".to_string()
    } else {
        result.push_str("\n  Fix these errors and run again.\n");
        result
    }
}

/// Extract // ZOR:L{line} comments from the generated Rust output
/// to build an accurate Rust-line → Zor-line mapping.
fn build_zor_line_map(rust_err: &str) -> std::collections::HashMap<usize, usize> {
    let mut map = std::collections::HashMap::new();
    // Look for // ZOR:L comments that rustc may have preserved in error context
    for line in rust_err.lines() {
        // rustc sometimes shows the source line in error messages
        if let Some(pos) = line.find("// ZOR:L") {
            if let Ok(n) = line[pos+8..].split_whitespace().next().unwrap_or("0").parse::<usize>() {
                // We know the Zor line, but need to know which Rust line it maps to
                // This is approximate — we track the mapping
                map.insert(n, n); // Store for later use
            }
        }
    }
    map
}

fn build_project(dir: &str) {
    let project_dir = Path::new(dir);
    if !project_dir.is_dir() {
        eprintln!("Zor: '{}' is not a directory", dir);
        std::process::exit(1);
    }
    
    let out_dir = project_dir.join("target").join("zor");
    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("Cannot create output dir");
    
    // Find all .zor files
    let mut zor_files = vec![];
    for entry in std::fs::read_dir(project_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "zor") {
            zor_files.push(path);
        }
    }
    
    if zor_files.is_empty() {
        eprintln!("Zor: no .zor files found in {}", dir);
        std::process::exit(1);
    }
    
    println!("Zor: building project with {} files", zor_files.len());
    
    // Transpile each file (with cross-file import resolution)
    let mut has_main = false;
    let mut bin_names = vec![];
    for zor_file in &zor_files {
        let name = zor_file.file_stem().unwrap().to_str().unwrap();
        let src = std::fs::read_to_string(zor_file).expect("Cannot read source");
        let mut stmts = parse_safe(&src, &format!("{}", zor_file.display()));
        
        // Resolve get imports from sibling .zor files
        let mut resolved = Vec::new();
        for s in stmts {
            if let ast::Stmt::Get(module, _) = &s {
                let mod_path = project_dir.join(format!("{}.zor", module));
                if mod_path.exists() {
                    let mod_src = std::fs::read_to_string(&mod_path).unwrap_or_default();
                    let mod_stmts = parse_safe(&mod_src, &format!("{}.zor", module));
                    resolved.extend(mod_stmts);
                    continue;
                }
            }
            resolved.push(s);
        }
        
        let resolved = monomorphize(resolved);
        
        // Check if this file has a main function (after import resolution)
        if resolved.iter().any(|s| matches!(s, ast::Stmt::Fun(n, _, _, _, _) if n == "main")) {
            has_main = true;
            bin_names.push(name.to_string());
        }
        
        if let Err(errs) = ownership::check(&resolved) {
            eprintln!("Zor: ownership errors in {}:", zor_file.display());
            for e in errs { eprintln!("  {}", e); }
            std::process::exit(1);
        }
        
        // Generate Cargo.toml with dependencies from use statements
        let mut cargo_deps = vec![];
        for s in &resolved {
            if let ast::Stmt::Use(path) = s {
                let crate_name = path.split("::").next().unwrap_or(path);
                if !cargo_deps.contains(&crate_name) {
                    cargo_deps.push(crate_name);
                }
            }
        }
        if !cargo_deps.is_empty() {
            let project_name = project_dir.file_name().unwrap().to_str().unwrap();
            let cargo_toml = format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{}\n",
                project_name,
                cargo_deps.iter().map(|d| format!("{} = \"*\"", d)).collect::<Vec<_>>().join("\n")
            );
            std::fs::write("Cargo.toml", &cargo_toml).ok();
        }

        let no_std = std::env::var("ZOR_NO_STD").is_ok();
        let mut gen = codegen_rust::RustGen::new(no_std);
        let mut rust_code = gen.generate(&resolved);
        
        // If we imported modules, add Rust mod declarations at the top
        if rust_code.contains("// Generated by Zor") || rust_code.contains("// Zor → Rust") {
            let mut imports = String::new();
            for s in &resolved {
                if let ast::Stmt::Get(module, _) = s {
                    // Check if it's a sibling .zor file
                    if project_dir.join(format!("{}.zor", module)).exists() {
                        imports.push_str(&format!("mod {};\nuse {}::*;\n", module, module));
                    }
                }
            }
            if !imports.is_empty() {
                // Insert after the first newline (after the header)
                if let Some(pos) = rust_code.find('\n') {
                    rust_code.insert_str(pos + 1, &imports);
                }
            }
        }
        
        let rs_file = src_dir.join(format!("{}.rs", name));
        std::fs::write(&rs_file, &rust_code).expect("Cannot write Rust output");
        println!("  {} → src/{}.rs", zor_file.display(), name);
    }
    
    // Generate Cargo.toml
    let project_name = project_dir.file_name().unwrap().to_str().unwrap();
    let mut cargo_toml = format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n",
        project_name
    );
    
    // Add bin targets for files with main
    for bin_name in &bin_names {
        cargo_toml.push_str(&format!("\n[[bin]]\nname = \"{}\"\npath = \"src/{}.rs\"\n", bin_name, bin_name));
    }
    
    std::fs::write(out_dir.join("Cargo.toml"), &cargo_toml).expect("Cannot write Cargo.toml");
    
    // Copy files to src/ layout if needed, or build in-place
    println!("Zor: generated Cargo project in {}/", out_dir.display());
    println!("Zor: run: cd {} && cargo build", out_dir.display());
    
    // Optionally auto-build
    let status = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(&out_dir)
        .status();
    
    match status {
        Ok(s) if s.success() => println!("Zor: cargo build succeeded!"),
        Ok(_) => eprintln!("Zor: cargo build failed"),
        Err(_) => println!("Zor: cargo not found — run manually"),
    }
}

// Generics Monomorphization

fn monomorphize(stmts: Vec<ast::Stmt>) -> Vec<ast::Stmt> {
    use ast::*;
    use std::collections::HashMap;
    let mut generic_funs: HashMap<String, (Vec<String>, Vec<Param>, Option<Type>, Vec<Stmt>)> = HashMap::new();
    for s in &stmts {
        if let Stmt::Fun(name, tps, params, ret, body) = s {
            if !tps.is_empty() { generic_funs.insert(name.clone(), (tps.clone(), params.clone(), ret.clone(), body.clone())); }
        }
    }
    if generic_funs.is_empty() { return stmts; }
    let mut specialized = Vec::new();
    let mut result = Vec::new();
    for s in stmts {
        match &s {
            Stmt::Fun(name, tps, _, _, _) => { if tps.is_empty() { result.push(s); } }
            Stmt::ExprStmt(Expr::Call(name, args)) | Stmt::Var(_, _, Some(Expr::Call(name, args))) => {
                if let Some((tps, params, ret, body)) = generic_funs.get(name) {
                    if tps.len() <= args.len() {
                        // Build substitution map: type_param → concrete_type
                        let mut subs: HashMap<String, Type> = HashMap::new();
                        let mut ok = true;
                        for (i, tp) in tps.iter().enumerate() {
                            if i < args.len() {
                                let ct = infer_type_from_expr(&args[i]);
                                subs.insert(tp.clone(), ct);
                            } else { ok = false; }
                        }
                        if ok {
                            let type_str = tps.iter().map(|tp| type_to_string(subs.get(tp).unwrap_or(&&ast::Type::Int))).collect::<Vec<_>>().join("_");
                            let nn = format!("{}_{}", name, type_str);
                            if !specialized.contains(&nn) {
                                specialized.push(nn.clone());
                                let sb = sub_body_map(&body, &subs);
                                let sp: Vec<Param> = params.iter().map(|p| Param { name: p.name.clone(), ty: sub_ty_map(&p.ty, &subs) }).collect();
                                let sr = ret.as_ref().map(|r| sub_ty_map(r, &subs));
                                result.push(Stmt::Fun(nn.clone(), vec![], sp, sr, sb));
                            }
                            result.push(match &s { Stmt::ExprStmt(_) => Stmt::ExprStmt(Expr::Call(nn, args.clone())), Stmt::Var(n, t, _) => Stmt::Var(n.clone(), t.clone(), Some(Expr::Call(nn, args.clone()))), _ => s });
                            continue;
                        }
                    }
                    result.push(s);
                } else { result.push(s); }
            }
            _ => result.push(s),
        }
    }
    result
}

fn infer_type_from_expr(e: &ast::Expr) -> ast::Type {
    match e {
        ast::Expr::Int(_) => ast::Type::Int,
        ast::Expr::Float(_) => ast::Type::Float,
        ast::Expr::Str(_) => ast::Type::String,
        ast::Expr::Bool(_) => ast::Type::Bool,
        ast::Expr::Var(name) => {
            // Could look up variable types but that requires context
            // For now, infer from naming convention
            if name.contains("int") || name.contains("num") { ast::Type::Int }
            else if name.contains("float") { ast::Type::Float }
            else if name.contains("str") || name.contains("name") { ast::Type::String }
            else if name.contains("bool") || name.starts_with("is_") { ast::Type::Bool }
            else { ast::Type::Int }
        },
        ast::Expr::StructLit(name, _) => ast::Type::Named(name.clone()),
        _ => ast::Type::Int,
    }
}
fn infer_type(e: &ast::Expr) -> ast::Type { infer_type_from_expr(e) }
fn type_to_string(t: &ast::Type) -> String { match t { ast::Type::Int => "int".into(), ast::Type::Float => "float".into(), ast::Type::String => "string".into(), ast::Type::Bool => "bool".into(), ast::Type::Named(s) => s.clone(), ast::Type::Ref(inner, is_mut) => format!("&{} {}", if *is_mut {"mut"} else {""}, type_to_string(inner)), _ => "unknown".into() } }

// Multi-param substitution using HashMap
fn sub_ty_map(t: &ast::Type, subs: &std::collections::HashMap<String, ast::Type>) -> ast::Type {
    match t {
        ast::Type::Named(s) | ast::Type::Generic(s) if subs.contains_key(s) => subs.get(s).cloned().unwrap_or(ast::Type::Int),
        ast::Type::Ptr(inner) => ast::Type::Ptr(Box::new(sub_ty_map(inner, subs))),
        ast::Type::Ref(inner, is_mut) => ast::Type::Ref(Box::new(sub_ty_map(inner, subs)), *is_mut),
        ast::Type::VecP(inner) => ast::Type::VecP(Box::new(sub_ty_map(inner, subs))),
        ast::Type::MapP(k, v) => ast::Type::MapP(Box::new(sub_ty_map(k, subs)), Box::new(sub_ty_map(v, subs))),
        _ => t.clone(),
    }
}
fn sub_body_map(stmts: &[ast::Stmt], subs: &std::collections::HashMap<String, ast::Type>) -> Vec<ast::Stmt> {
    stmts.iter().map(|s| sub_stmt_map(s, subs)).collect()
}
fn sub_stmt_map(s: &ast::Stmt, subs: &std::collections::HashMap<String, ast::Type>) -> ast::Stmt {
    use ast::*;
    match s {
        Stmt::Var(n, t, v) => Stmt::Var(n.clone(), sub_ty_map(t, subs), v.as_ref().map(|e| sub_expr_map(e, subs))),
        Stmt::Assign(n, e) => Stmt::Assign(n.clone(), sub_expr_map(e, subs)),
        Stmt::Say(e) => Stmt::Say(sub_expr_map(e, subs)),
        Stmt::Give(e) => Stmt::Give(e.as_ref().map(|e| sub_expr_map(e, subs))),
        Stmt::If(cs, el) => Stmt::If(cs.iter().map(|(c,b)| (sub_expr_map(c,subs), sub_body_map(b,subs))).collect(), el.as_ref().map(|b| sub_body_map(b,subs))),
        Stmt::Repeat(k, b) => Stmt::Repeat(match k { RepKind::While(c) => RepKind::While(sub_expr_map(c,subs)), RepKind::Until(c) => RepKind::Until(sub_expr_map(c,subs)), RepKind::Count(c) => RepKind::Count(sub_expr_map(c,subs)), RepKind::Range(v, s, e) => RepKind::Range(v.clone(), Box::new(sub_expr_map(s,subs)), Box::new(sub_expr_map(e,subs))), other => other.clone() }, sub_body_map(b,subs)),
        Stmt::ExprStmt(e) => Stmt::ExprStmt(sub_expr_map(e, subs)),
        Stmt::Fun(n, _, p, r, b) => Stmt::Fun(n.clone(), vec![], p.iter().map(|p| Param{name:p.name.clone(), ty:sub_ty_map(&p.ty,subs)}).collect(), r.as_ref().map(|r| sub_ty_map(r,subs)), sub_body_map(b,subs)),
        Stmt::FieldAssign(obj, f, v) => Stmt::FieldAssign(Box::new(sub_expr_map(obj, subs)), f.clone(), sub_expr_map(v, subs)),
        Stmt::Extern(fns) => Stmt::Extern(fns.clone()),
        Stmt::Unsafe(body) => Stmt::Unsafe(sub_body_map(body, subs)),
        Stmt::Use(path) => Stmt::Use(path.clone()),
        _ => s.clone(),
    }
}
fn sub_expr_map(e: &ast::Expr, subs: &std::collections::HashMap<String, ast::Type>) -> ast::Expr {
    use ast::*;
    match e {
        Expr::Bin(l,op,r) => Expr::Bin(Box::new(sub_expr_map(l,subs)), op.clone(), Box::new(sub_expr_map(r,subs))),
        Expr::Una(op,e) => Expr::Una(op.clone(), Box::new(sub_expr_map(e,subs))),
        Expr::Call(n,args) => Expr::Call(n.clone(), args.iter().map(|a| sub_expr_map(a,subs)).collect()),
        Expr::MethodCall(obj,m,args) => Expr::MethodCall(Box::new(sub_expr_map(obj,subs)), m.clone(), args.iter().map(|a| sub_expr_map(a,subs)).collect()),
        Expr::Closure(ps,ret,body) => Expr::Closure(ps.iter().map(|p| Param{name:p.name.clone(), ty:sub_ty_map(&p.ty,subs)}).collect(), ret.as_ref().map(|r| sub_ty_map(r,subs)), sub_body_map(body,subs)),
        Expr::Try(e) => Expr::Try(Box::new(sub_expr_map(e,subs))),
        Expr::Path(segs) => Expr::Path(segs.clone()),
        Expr::Field(obj, f) => Expr::Field(Box::new(sub_expr_map(obj, subs)), f.clone()),
        Expr::New(e) => Expr::New(Box::new(sub_expr_map(e, subs))),
        Expr::Free(e) => Expr::Free(Box::new(sub_expr_map(e, subs))),
        Expr::Deref(e) => Expr::Deref(Box::new(sub_expr_map(e, subs))),
        Expr::AddrOf(e) => Expr::AddrOf(Box::new(sub_expr_map(e, subs))),
        Expr::AddrOfMut(e) => Expr::AddrOfMut(Box::new(sub_expr_map(e, subs))),
        Expr::StructLit(n, fields) => Expr::StructLit(n.clone(), fields.iter().map(|(f,v)| (f.clone(), sub_expr_map(v, subs))).collect()),
        _ => e.clone(),
    }
}

// Keep old single-param versions for backward compat
fn sub_ty(t: &ast::Type, f: &str, to: &ast::Type) -> ast::Type {
    let mut m = std::collections::HashMap::new();
    m.insert(f.to_string(), to.clone());
    sub_ty_map(t, &m)
}
fn sub_body(stmts: &[ast::Stmt], f: &str, to: &ast::Type) -> Vec<ast::Stmt> {
    let mut m = std::collections::HashMap::new();
    m.insert(f.to_string(), to.clone());
    sub_body_map(stmts, &m)
}
fn sub_stmt(s: &ast::Stmt, f: &str, to: &ast::Type) -> ast::Stmt {
    let mut m = std::collections::HashMap::new();
    m.insert(f.to_string(), to.clone());
    sub_stmt_map(s, &m)
}
fn sub_expr(e: &ast::Expr, f: &str, to: &ast::Type) -> ast::Expr {
    let mut m = std::collections::HashMap::new();
    m.insert(f.to_string(), to.clone());
    sub_expr_map(e, &m)
}

// Package Management

fn find_latest_version(pkg_dir: &Path) -> Option<PathBuf> {
    let mut versions: Vec<_> = std::fs::read_dir(pkg_dir).ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();
    versions.sort();
    versions.pop()
}

fn pkg_init(name: &str) {
    let manifest = format!("{}.toml", name);
    if Path::new(&manifest).exists() {
        eprintln!("Zor: {} already exists", manifest);
        return;
    }
    let toml = format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\ndescription = \"A Zor package\"\n\n[dependencies]\n", name);
    std::fs::write(&manifest, &toml).unwrap();
    println!("Zor: created {}", manifest);
    let src = format!("// {} - v0.1.0\n\nfun hello() string = \"Hello from {}!\"\n", name, name);
    let src_file = format!("{}.zor", name);
    std::fs::write(&src_file, &src).unwrap();
    println!("Zor: created {}", src_file);
}

fn pkg_add(name: &str) {
    let cache = pkg_cache_dir();
    std::fs::create_dir_all(&cache).ok();
    
    let registry = Path::new("registry");
    let pkg_dir = registry.join(name);
    
    if pkg_dir.is_dir() {
        if let Some(latest) = find_latest_version(&pkg_dir) {
            let version = latest.file_name().unwrap().to_str().unwrap();
            let dest = cache.join(name).join(version);
            if !dest.exists() {
                copy_dir(&latest, &dest).ok();
                println!("Zor: added {} v{}", name, version);
            } else {
                println!("Zor: {} v{} already installed", name, version);
            }
        }
    } else {
        let dest = cache.join(name).join("0.1.0");
        if !dest.exists() {
            std::fs::create_dir_all(&dest).unwrap();
            let stub = format!("// Package: {} v0.1.0\nfun hello() string = \"Hello from {}!\"\n", name, name);
            std::fs::write(dest.join(format!("{}.zor", name)), &stub).unwrap();
            println!("Zor: added {} v0.1.0", name);
        } else {
            println!("Zor: {} already installed", name);
        }
    }
}

fn pkg_install() {
    let manifest = Path::new("Zor.toml");
    if !manifest.exists() {
        eprintln!("Zor: no Zor.toml found. Run 'zor pkg init <name>' first.");
        return;
    }
    let content = std::fs::read_to_string(manifest).unwrap();
    let mut in_deps = false;
    for line in content.lines() {
        let line = line.trim();
        if line == "[dependencies]" { in_deps = true; continue; }
        if line.starts_with('[') { in_deps = false; continue; }
        if in_deps && !line.is_empty() && !line.starts_with('#') {
            let name = line.split('=').next().unwrap().trim().trim_matches('"');
            if !name.is_empty() {
                println!("Zor: installing {}...", name);
                pkg_add(name);
            }
        }
    }
}

fn pkg_publish() {
    let manifest = Path::new("Zor.toml");
    if !manifest.exists() {
        eprintln!("Zor: no Zor.toml found. Run 'zor pkg init <name>' first.");
        return;
    }
    let content = std::fs::read_to_string(manifest).unwrap();
    let name = content.lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("unknown");
    let version = content.lines()
        .find(|l| l.starts_with("version"))
        .and_then(|l| l.split('"').nth(1))
        .unwrap_or("0.1.0");
    
    let registry = Path::new("registry");
    let dest = registry.join(name).join(version);
    std::fs::create_dir_all(&dest).unwrap();
    
    // Copy manifest
    std::fs::copy(manifest, dest.join("Zor.toml")).unwrap();
    // Copy source files
    let src_file = format!("{}.zor", name);
    if Path::new(&src_file).exists() {
        std::fs::copy(&src_file, dest.join(&src_file)).unwrap();
    }
    // Copy any additional .zor files
    for entry in std::fs::read_dir(".").into_iter().flatten() {
        if let Ok(e) = entry {
            let p = e.path();
            if p.extension().map_or(false, |x| x == "zor") && p.file_stem().unwrap().to_str().unwrap() != name {
                std::fs::copy(&p, dest.join(p.file_name().unwrap())).ok();
            }
        }
    }
    println!("Zor: published {} v{} to registry/", name, version);
}

fn copy_dir(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

// CLI

fn self_update() {
    let repo_url = "https://github.com/xytrolabs/zor.git";
    let tmp = std::env::temp_dir().join("zor-update");
    
    println!("⚡ Zor updater — fetching latest from GitHub...");
    
    // Clone or pull the repo
    let git_result = if tmp.join(".git").exists() {
        Command::new("git").args(&["-C", tmp.to_str().unwrap_or("/tmp/zor-update"), "pull", "--ff-only"])
            .output()
    } else {
        let _ = std::fs::remove_dir_all(&tmp);
        Command::new("git").args(&["clone", "--depth", "1", repo_url, tmp.to_str().unwrap_or("/tmp/zor-update")])
            .output()
    };
    
    match git_result {
        Ok(out) if out.status.success() => {},
        Ok(out) => {
            eprintln!("Zor: git failed — {}\n{}", 
                String::from_utf8_lossy(&out.stderr),
                "Make sure git is installed and you have internet access.");
            std::process::exit(1);
        },
        Err(e) => {
            eprintln!("Zor: git not found — install git to use auto-update.\n  {}", e);
            std::process::exit(1);
        }
    }
    
    println!("  ✓ Repository up to date");
    println!("  Building with cargo...");
    
    let build = Command::new("cargo")
        .args(&["build", "--release"])
        .current_dir(tmp.join("zor-native"))
        .output();
    
    match build {
        Ok(out) if out.status.success() => {},
        Ok(out) => {
            eprintln!("Zor: cargo build failed — {}\n{}",
                String::from_utf8_lossy(&out.stderr),
                "Make sure Rust is installed: https://rustup.rs");
            std::process::exit(1);
        },
        Err(e) => {
            eprintln!("Zor: cargo not found — install Rust: https://rustup.rs\n  {}", e);
            std::process::exit(1);
        }
    }
    
    println!("  ✓ Build succeeded");
    
    // Copy the binary over the current one
    let current = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("zor"));
    let new_bin = tmp.join("zor-native/target/release/zor");
    
    if new_bin.exists() {
        // Back up the current binary
        let backup = format!("{}.bak", current.display());
        std::fs::copy(&current, &backup).ok();
        
        match std::fs::copy(&new_bin, &current) {
            Ok(_) => {
                println!("  ✓ Updated to the latest Zor!");
                println!("  (Backup saved to {})", backup);
            },
            Err(e) => {
                eprintln!("Zor: cannot replace binary — try running with sudo.\n  {}", e);
                eprintln!("  New binary at: {}", new_bin.display());
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Zor: built binary not found at {}", new_bin.display());
        std::process::exit(1);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Zor v0.5.0 — Xytro Labs (transpiles to Rust)");
        eprintln!("  zor run <file.zor>       Compile & run");
        eprintln!("  zor build <file.zor>     Compile to native binary (via rustc)");
        eprintln!("  zor check <file.zor>     Parse & check only");
        eprintln!("  zor test <file.zor>      Run test_* functions");
        eprintln!("  zor doc <file.zor>       Generate documentation");
        eprintln!("  zor fmt <file.zor>       Format source code");
        eprintln!("  zor --update             Update to latest version");
        eprintln!("  zor pkg init <name>      Create new package");
        eprintln!("  zor pkg add <name>       Install a package");
        eprintln!("  zor pkg install          Install from Zor.toml");
        eprintln!("  zor pkg publish          Publish to registry/");
        eprintln!();
        eprintln!("  ZOR_NO_STD=1 zor build    Build without std (for OS dev)");
        return;
    }
    let cmd = &args[1];
    
    if cmd == "--update" || cmd == "update" {
        self_update();
        return;
    }
    
    if cmd == "pkg" {
        if args.len() < 3 {
            eprintln!("Zor: pkg needs subcommand: init, add, install");
            return;
        }
        match args[2].as_str() {
            "init" => { if let Some(name) = args.get(3) { pkg_init(name); } else { eprintln!("Zor: Usage: zor pkg init <name>"); } },
            "add" => { if let Some(name) = args.get(3) { pkg_add(name); } else { eprintln!("Zor: Usage: zor pkg add <name>"); } },
            "install" => pkg_install(),
            "publish" => pkg_publish(),
            _ => eprintln!("Zor: unknown pkg command: {}", args[2]),
        }
        return;
    }
    
    if args.len() < 3 {
        eprintln!("Zor: need a file. Usage: zor {} <file.zor> [--target <triple>]", cmd);
        return;
    }
    let input = &args[2];
    let target = args.iter().position(|a| a == "--target")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str())
        .unwrap_or("");
    match cmd.as_str() {
        "run" => {
            if Path::new(input).is_dir() {
                build_project(input);
            } else {
                let out = Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("a");
                compile_zor(input, out, target);
                let s = Command::new(format!("./{}", out)).status().unwrap_or_else(|_| std::process::ExitStatus::default());
                std::process::exit(s.code().unwrap_or(1));
            }
        },
        "build" => {
            if Path::new(input).is_dir() {
                build_project(input);
            } else {
                let out = Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("a");
                compile_zor(input, out, target);
            }
        },
        "check" => { let src = std::fs::read_to_string(input).unwrap_or_default(); let s = parse_safe(&src, input);
            println!("Zor: OK - {} statements",s.len());
            for st in &s { match st {
                ast::Stmt::Fun(n,_,ps,ret,_) => { let p:Vec<String>=ps.iter().map(|p|format!("{}: {:?}",p.name,p.ty)).collect(); println!("  fun {}({}) -> {:?}",n,p.join(", "),ret); },
                ast::Stmt::StructDef(n, _, fs) => { println!("  struct {} {{ {} }}", n, fs.iter().map(|f| format!("{}: {:?}", f.name, f.ty)).collect::<Vec<_>>().join(", ")); },
                ast::Stmt::TraitDef(n, ms) => { println!("  trait {} {{ {} methods }}", n, ms.len()); },
                ast::Stmt::ImplDef(tn, ty, body) => { println!("  impl {} for {} ({} methods)", tn, ty, body.len()); },
                _ => {},
            }}
        },
        "test" => run_tests(input),
        "doc" => generate_docs(input),
        "fmt" => {
            let src = std::fs::read_to_string(input).unwrap_or_default();
            let stmts = parse_safe(&src, input);
            let formatted = fmt::format(&stmts);
            if std::fs::write(input, &formatted).is_ok() {
                println!("Zor: formatted {}", input);
            } else {
                eprintln!("Zor: cannot write {}", input);
            }
        },
        _ => eprintln!("Unknown: {}",cmd),
    }
}

fn generate_docs(input: &str) {
    let src = std::fs::read_to_string(input).unwrap_or_default();
    let name = Path::new(input).file_stem().and_then(|s| s.to_str()).unwrap_or("docs");
    let mut html = format!("<!DOCTYPE html>\n<html><head><title>{} - Zor Docs</title>\n<style>
body{{font-family:system-ui,sans-serif;max-width:900px;margin:0 auto;padding:20px;background:#0d1117;color:#c9d1d9}}
h1{{color:#58a6ff;border-bottom:1px solid #30363d;padding-bottom:8px}}
h2{{color:#f0883e}}
.sig{{background:#161b22;padding:8px 12px;border-radius:6px;font-family:monospace;color:#7ee787}}
.doc{{color:#8b949e;margin:8px 0 16px 16px}}
.tag{{display:inline-block;background:#1f6feb22;color:#58a6ff;padding:2px 8px;border-radius:4px;font-size:12px;margin-right:4px}}
</style></head><body>
<h1>📦 {}</h1>\n", name, name);

    let lines: Vec<&str> = src.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        // Collect /// doc comments
        if line.starts_with("///") || line.starts_with("//!") {
            let mut doc = String::new();
            while i < lines.len() && (lines[i].trim().starts_with("///") || lines[i].trim().starts_with("//!")) {
                let d = lines[i].trim().trim_start_matches("///").trim_start_matches("//!").trim();
                if !doc.is_empty() { doc.push(' '); }
                doc.push_str(d);
                i += 1;
            }
            // Next non-empty line should be a definition
            while i < lines.len() && lines[i].trim().is_empty() { i += 1; }
            if i < lines.len() {
                let def = lines[i].trim();
                if def.starts_with("fun ") {
                    let sig = def.trim_end_matches(" {").trim_end_matches('{').to_string();
                    html.push_str(&format!("<h2><span class=\"tag\">fun</span> {}</h2>\n", &sig[4..]));
                    html.push_str(&format!("<div class=\"sig\">{}</div>\n", html_escape(&sig)));
                    html.push_str(&format!("<div class=\"doc\">{}</div>\n", doc));
                } else if def.starts_with("struct ") {
                    let sig = def.trim_end_matches(" {").trim_end_matches('{').to_string();
                    html.push_str(&format!("<h2><span class=\"tag\">struct</span> {}</h2>\n", &sig[7..]));
                    html.push_str(&format!("<div class=\"sig\">{}</div>\n", html_escape(&sig)));
                    html.push_str(&format!("<div class=\"doc\">{}</div>\n", doc));
                } else if def.starts_with("kind ") {
                    let sig = def.trim_end_matches(" {").trim_end_matches('{').to_string();
                    html.push_str(&format!("<h2><span class=\"tag\">kind</span> {}</h2>\n", &sig[5..]));
                    html.push_str(&format!("<div class=\"sig\">{}</div>\n", html_escape(&sig)));
                    html.push_str(&format!("<div class=\"doc\">{}</div>\n", doc));
                } else if def.starts_with("trait ") {
                    let sig = def.trim_end_matches(" {").trim_end_matches('{').to_string();
                    html.push_str(&format!("<h2><span class=\"tag\">trait</span> {}</h2>\n", &sig[6..]));
                    html.push_str(&format!("<div class=\"sig\">{}</div>\n", html_escape(&sig)));
                    html.push_str(&format!("<div class=\"doc\">{}</div>\n", doc));
                }
            }
        }
        i += 1;
    }
    html.push_str("</body></html>");
    let out = format!("{}.doc.html", name);
    std::fs::write(&out, &html).ok();
    println!("Zor: docs written to {}", out);
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn run_tests(input: &str) {
    let src = std::fs::read_to_string(input).unwrap_or_default();
    let stmts = parse_safe(&src, input);
    let mut tests = Vec::new();
    for s in &stmts {
        if let ast::Stmt::Fun(name, _, _, _, _) = s {
            if name.starts_with("test_") {
                tests.push(name.clone());
            }
        }
    }
    if tests.is_empty() {
        println!("Zor: no tests found (functions starting with test_)");
        return;
    }
    let mut passed = 0;
    let mut failed = 0;
    for test in &tests {
        let test_src = format!("{}\n{}()\n", &src, test);
        let tmp_file = format!("/tmp/zor_test_{}.zor", test);
        std::fs::write(&tmp_file, &test_src).ok();
        
        let out = format!("/tmp/zor_test_{}_bin", test);
        let status = std::panic::catch_unwind(|| {
            compile_zor(&tmp_file, &out, "");
            Command::new(&out).status().unwrap_or_else(|_| std::process::ExitStatus::default())
        });
        
        match status {
            Ok(s) if s.success() => { passed += 1; println!("  OK  {}", test); },
            _ => { failed += 1; println!("  FAIL {}", test); },
        }
        std::fs::remove_file(&tmp_file).ok();
        std::fs::remove_file(&out).ok();
        std::fs::remove_file(format!("{}.s", out)).ok();
    }
    println!("\nZor: {} passed, {} failed", passed, failed);
    if failed > 0 { std::process::exit(1); }
}
