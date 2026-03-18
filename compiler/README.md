# compiler/

QLN 2.0 `.n2` 언어 컴파일러 (Rust + pest PEG 파서).

## 실행
```bash
cargo build
cargo run -- examples/soul-boot.n2
```

## 구조
- `src/grammar.pest` — PEG 문법 정의
- `src/ast.rs` — AST 노드 구조체
- `src/parser.rs` — pest → AST 변환
- `src/main.rs` — CLI 엔트리포인트
- `examples/` — .n2 예제 파일
