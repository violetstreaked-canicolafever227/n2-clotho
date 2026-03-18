// n2c — .n2 언어 컴파일러 CLI (파싱 + 검증 + 계약 + 쿼리)
use std::env;
use std::fs;
use n2_compiler::parser::parse_n2;
use n2_compiler::validator;
use n2_compiler::contract::ContractRuntime;
use n2_compiler::query::N2Registry;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("🔧 n2c v0.3.0 — QLN 2.0 컴파일러");
        eprintln!();
        eprintln!("사용법:");
        eprintln!("  n2c <파일.n2>                           파싱 + AST JSON 출력");
        eprintln!("  n2c validate <파일.n2>                  파싱 + 검증 + 계약 체크");
        eprintln!("  n2c simulate <파일.n2>                  계약 상태머신 시뮬레이션");
        eprintln!("  n2c query <파일.n2> \"SELECT * FROM rules\"   SQL 쿼리");
        std::process::exit(1);
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

    println!("🔧 n2c v0.3.0 — QLN 2.0 컴파일러");
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
        "parse" => {
            let json = serde_json::to_string_pretty(&ast).unwrap();
            println!("✅ 파싱 성공! AST:");
            println!("{}", json);
            print_summary(&ast);
        }
        "validate" => {
            println!("── Step 1: 파싱 ✅");
            print_summary(&ast);

            // Step 2: 스키마 검증
            println!();
            println!("── Step 2: 스키마 검증");
            let errors = validator::validate(&ast);
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

            // Step 3: 계약 체크
            println!();
            println!("── Step 3: 계약 체크");
            let runtime = ContractRuntime::from_file(&ast);

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

            // 최종 결과
            println!();
            if error_count > 0 {
                println!("🚨 검증 실패: {} 에러 발견", error_count);
                std::process::exit(1);
            } else {
                println!("✅ 검증 완료: 모든 체크 통과!");
            }
        }
        "simulate" => {
            println!("── 계약 상태머신 시뮬레이션");
            let runtime = ContractRuntime::from_file(&ast);

            if runtime.machines.is_empty() {
                println!("  ℹ️ 상태머신 계약이 없습니다");
                std::process::exit(0);
            }

            println!("{}", runtime.summary());
        }
        "query" => {
            let registry = N2Registry::from_file(&ast);
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
        _ => {
            eprintln!("❌ 알 수 없는 명령: '{}'", command);
            eprintln!("   사용 가능: parse, validate, simulate, query");
            std::process::exit(1);
        }
    }
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
