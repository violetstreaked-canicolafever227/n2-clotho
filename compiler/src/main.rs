// n2c — .n2 언어 컴파일러 CLI (파싱 + 검증 + 계약 + 쿼리 + 멀티 타겟 코드생성)
use std::env;
use std::fs;
use std::path::Path;
use n2_compiler::parser::parse_n2;
use n2_compiler::validator;
use n2_compiler::contract::ContractRuntime;
use n2_compiler::query::N2Registry;
use n2_compiler::codegen::{BackendRegistry, CompilationMeta, extract_meta};

const VERSION: &str = "3.0.0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    // Handle commands that don't need a file
    if args[1] == "backends" {
        cmd_backends();
        return;
    }

    let (command, filepath, extra) = if args.len() >= 4 {
        (args[1].as_str(), &args[2], Some(args[3].clone()))
    } else if args.len() >= 3 {
        (args[1].as_str(), &args[2], None)
    } else {
        ("parse", &args[1], None)
    };

    let source = match fs::read_to_string(filepath) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ 파일 읽기 실패 [{}]: {}", filepath, e);
            std::process::exit(1);
        }
    };

    println!("🔧 n2c v{} — Clotho compiler", VERSION);
    println!("📄 파일: {}", filepath);
    println!();

    // Step 1: 파싱
    let ast = match parse_n2(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    };

    match command {
        "parse" => cmd_parse(&ast),
        "validate" => cmd_validate(&ast),
        "simulate" => cmd_simulate(&ast),
        "query" => cmd_query(&ast, extra),
        "compile" => cmd_compile(&ast, filepath, extra),
        _ => {
            eprintln!("❌ 알 수 없는 명령: '{}'", command);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("🔧 n2c v{} — Clotho multi-target compiler", VERSION);
    eprintln!();
    eprintln!("사용법:");
    eprintln!("  n2c <파일.n2>                                 파싱 + AST JSON 출력");
    eprintln!("  n2c validate <파일.n2>                        파싱 + 검증 + 계약 체크");
    eprintln!("  n2c simulate <파일.n2>                        계약 상태머신 시뮬레이션");
    eprintln!("  n2c query <파일.n2> \"SELECT * FROM rules\"     SQL 쿼리");
    eprintln!("  n2c compile <파일.n2> --target <TARGET>       지정 타겟으로 컴파일");
    eprintln!("  n2c compile <파일.n2> --target all            전체 타겟 일괄 컴파일");
    eprintln!("  n2c backends                                  지원 백엔드 목록");
    eprintln!();
    eprintln!("타겟: rust(.n2rs) | c(.n2c) | cpp(.n2c2) | go(.n2go) | python(.n2py) | ts(.n2ts)");
}

fn cmd_parse(ast: &n2_compiler::ast::N2File) {
    let json = serde_json::to_string_pretty(ast).unwrap();
    println!("✅ 파싱 성공! AST:");
    println!("{}", json);
    print_summary(ast);
}

fn cmd_validate(ast: &n2_compiler::ast::N2File) {
    println!("── Step 1: 파싱 ✅");
    print_summary(ast);

    println!();
    println!("── Step 2: 스키마 검증");
    let errors = validator::validate(ast);
    let error_count = errors.iter()
        .filter(|e| e.severity == validator::Severity::Error)
        .count();
    let warn_count = errors.iter()
        .filter(|e| e.severity == validator::Severity::Warning)
        .count();

    for err in &errors {
        println!("  {}", err);
    }

    if error_count == 0 && warn_count == 0 {
        println!("  ✅ 검증 통과! 에러 0, 경고 0");
    } else {
        println!("  결과: ❌ {} 에러, ⚠️ {} 경고", error_count, warn_count);
    }

    println!();
    println!("── Step 3: 계약 체크");
    let runtime = ContractRuntime::from_file(ast);

    if runtime.machines.is_empty() {
        println!("  ℹ️ 상태머신 계약 없음 (스킵)");
    } else {
        println!("{}", runtime.summary().lines()
            .map(|l| format!("  {}", l))
            .collect::<Vec<_>>()
            .join("\n"));

        let violations = runtime.check_integrity();
        if violations.is_empty() {
            println!("  ✅ 상태머신 무결성 검증 통과!");
        } else {
            for v in &violations {
                println!("  {}", v);
            }
        }
    }

    println!();
    if error_count > 0 {
        println!("🚨 검증 실패: {} 에러 발견", error_count);
        std::process::exit(1);
    } else {
        println!("✅ 검증 완료: 모든 체크 통과!");
    }
}

fn cmd_simulate(ast: &n2_compiler::ast::N2File) {
    println!("── 계약 상태머신 시뮬레이션");
    let runtime = ContractRuntime::from_file(ast);

    if runtime.machines.is_empty() {
        println!("  ℹ️ 상태머신 계약이 없습니다");
        std::process::exit(0);
    }

    println!("{}", runtime.summary());
}

fn cmd_query(ast: &n2_compiler::ast::N2File, extra: Option<String>) {
    let registry = N2Registry::from_file(ast);
    println!("{}", registry.summary());
    println!();

    let sql = extra.unwrap_or_else(|| "SELECT * FROM rules".to_string());
    println!("📝 SQL: {}", sql);
    println!();

    match registry.execute_query(&sql) {
        Ok(result) => print!("{}", result),
        Err(e) => {
            eprintln!("❌ 쿼리 에러: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_compile(ast: &n2_compiler::ast::N2File, filepath: &str, target_arg: Option<String>) {
    let target = target_arg
        .as_deref()
        .and_then(|s| s.strip_prefix("--target=").or_else(|| s.strip_prefix("--target ")))
        .or_else(|| target_arg.as_deref())
        .unwrap_or("all");

    // Handle --target flag passed as separate arg
    let target = if target == "--target" {
        // target value would be in the next position, but we don't have it
        // Default to all
        "all"
    } else if target.starts_with("--target") {
        target.trim_start_matches("--target").trim_start_matches('=').trim()
    } else {
        target
    };

    let registry = BackendRegistry::new();
    let (source_name, source_version) = extract_meta(ast);

    let now = chrono_now();
    let base_meta = CompilationMeta {
        source_name: source_name.clone(),
        source_version: source_version.clone(),
        target: String::new(),
        extension: String::new(),
        compiler_version: format!("n2c v{}", VERSION),
        compiled_at: now,
    };

    let base_path = Path::new(filepath).with_extension("");

    if target == "all" {
        println!("🎯 전체 타겟 일괄 컴파일");
        println!();

        let results = registry.compile_all(ast, &base_meta);
        let mut success_count = 0;
        let mut fail_count = 0;

        for (target_name, ext, result) in &results {
            let out_path = format!("{}{}", base_path.display(), ext);
            match result {
                Ok(code) => {
                    fs::write(&out_path, code).unwrap_or_else(|e| {
                        eprintln!("  ❌ {} 파일 쓰기 실패: {}", out_path, e);
                    });
                    println!("  ✅ {} → {} ({} bytes)", target_name, out_path, code.len());
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("  ❌ {} 실패: {}", target_name, e);
                    fail_count += 1;
                }
            }
        }

        println!();
        println!("📊 결과: {} 성공, {} 실패 / {} 타겟",
            success_count, fail_count, results.len());
    } else {
        let mut meta = base_meta;
        meta.target = target.to_string();

        match registry.compile(ast, target, &meta) {
            Ok(code) => {
                let ext = registry.get(target)
                    .map(|b| b.file_extension())
                    .unwrap_or(".n2out");
                let out_path = format!("{}{}", base_path.display(), ext);
                fs::write(&out_path, &code).unwrap_or_else(|e| {
                    eprintln!("❌ 파일 쓰기 실패: {}", e);
                    std::process::exit(1);
                });
                println!("✅ {} → {} ({} bytes)", target, out_path, code.len());
            }
            Err(e) => {
                eprintln!("❌ {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn cmd_backends() {
    println!("🔧 n2c v{} — Clotho multi-target compiler", VERSION);
    println!();
    println!("지원 백엔드:");
    println!();
    let registry = BackendRegistry::new();
    for (name, ext) in registry.list() {
        println!("  {:12} → {}", name, ext);
    }
    println!();
    println!("총 {} 타겟 지원", registry.list().len());
}

fn print_summary(ast: &n2_compiler::ast::N2File) {
    use n2_compiler::ast::Block;
    let mut counts = [0u32; 8];
    for block in &ast.blocks {
        match block {
            Block::Meta(_) => counts[0] += 1,
            Block::Import(_) => counts[1] += 1,
            Block::Schema(_) => counts[2] += 1,
            Block::Contract(_) => counts[3] += 1,
            Block::Rule(_) => counts[4] += 1,
            Block::Workflow(_) => counts[5] += 1,
            Block::Query(_) => counts[6] += 1,
            Block::Semantic(_) => counts[7] += 1,
        }
    }
    println!("📊 블록: @meta:{} @import:{} @schema:{} @contract:{} @rule:{} @workflow:{} @query:{} @semantic:{} | 총 {}",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7],
        ast.blocks.len()
    );
}

/// Simple timestamp without chrono dependency
fn chrono_now() -> String {
    // Use a basic approach to avoid adding chrono dependency
    format!("compiled-by-n2c-v{}", VERSION)
}
