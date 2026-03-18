// n2c — .n2 언어 컴파일러 CLI
use std::env;
use std::fs;
use n2_compiler::parser::parse_n2;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("사용법: n2c <파일.n2>");
        eprintln!("  예: n2c examples/soul-boot.n2");
        std::process::exit(1);
    }

    let filepath = &args[1];
    let source = match fs::read_to_string(filepath) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ 파일 읽기 실패 [{}]: {}", filepath, e);
            std::process::exit(1);
        }
    };

    println!("🔧 n2c v0.1.0 — QLN 2.0 컴파일러");
    println!("📄 파싱 중: {}", filepath);
    println!();

    match parse_n2(&source) {
        Ok(ast) => {
            let json = serde_json::to_string_pretty(&ast).unwrap();
            println!("✅ 파싱 성공! AST:");
            println!("{}", json);

            // 요약 통계
            let mut meta = 0;
            let mut imports = 0;
            let mut schemas = 0;
            let mut contracts = 0;
            let mut rules = 0;
            let mut workflows = 0;
            let mut queries = 0;
            let mut semantics = 0;

            for block in &ast.blocks {
                match block {
                    n2_compiler::ast::Block::Meta(_) => meta += 1,
                    n2_compiler::ast::Block::Import(_) => imports += 1,
                    n2_compiler::ast::Block::Schema(_) => schemas += 1,
                    n2_compiler::ast::Block::Contract(_) => contracts += 1,
                    n2_compiler::ast::Block::Rule(_) => rules += 1,
                    n2_compiler::ast::Block::Workflow(_) => workflows += 1,
                    n2_compiler::ast::Block::Query(_) => queries += 1,
                    n2_compiler::ast::Block::Semantic(_) => semantics += 1,
                }
            }

            println!();
            println!("📊 블록 요약:");
            println!("  @meta: {}  @import: {}  @schema: {}", meta, imports, schemas);
            println!("  @contract: {}  @rule: {}  @workflow: {}", contracts, rules, workflows);
            println!("  @query: {}  @semantic: {}", queries, semantics);
            println!("  총 {} 블록", ast.blocks.len());
        }
        Err(e) => {
            eprintln!("❌ {}", e);
            std::process::exit(1);
        }
    }
}
