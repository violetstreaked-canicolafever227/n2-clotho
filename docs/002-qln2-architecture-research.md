# 002 — QLN 2.0 아키텍처 리서치

> 작성: 로제 | 2026-03-18 | Phase 1: 기술 리서치

---

## 1. 선행 기술 분석

### 1.1 Typst — Rust 기반 마크업 프로그래밍 언어 ⭐⭐⭐⭐⭐

> **가장 가까운 선행 사례.** Rust로 작성된 마크업 언어로서 참조 가치가 높다.

- **핵심**: Rust 컴파일러가 마크업 + 프로그래밍 코드를 AST로 파싱 → 구조화된 출력 생성
- **특징**: 변수, 함수, 조건문, 루프 지원 — 순수 마크업을 넘어 프로그래밍 가능
- **성능**: v0.12부터 멀티스레드 레이아웃 엔진 지원
- **접근성**: PDF/UA-1 표준 지원, 태그된 PDF 기본 생성
- **시사점**:
  - `.n2` 파일도 **마크업(가독성) + 프로그래밍(강제성)** 하이브리드로 설계 가능
  - Typst의 스크립팅 시스템 참조: `#let`, `#if`, `#for` 등 마크업 내 코드 삽입 패턴
  - AST 구조와 타입 시스템을 `.n2` 언어에 적용

```typst
// Typst 예시 — 마크업 + 프로그래밍 혼합
#let agent = "rose"
#if agent == "rose" [
  ## 부팅 시퀀스
  - n2_boot(agent: #agent)
  - 핸드오프 확인
]
```

### 1.2 MDX — Markdown + JSX 하이브리드 ⭐⭐⭐⭐

> MD에 프로그래밍 기능을 결합한 대표 사례. 컴파일 파이프라인이 핵심.

- **파이프라인**: `.mdx` → mdast(MD AST) → hast(HTML AST) → esast(JS AST) → JavaScript
- **unified 생태계**: remark(mdast) → rehype(hast) → recma(esast) 플러그인 체인
- **컴파일 타임**: 런타임 없이 빌드 시 모두 컴파일 → 성능 우수
- **시사점**:
  - `.n2` 컴파일러도 **다단계 AST 변환 파이프라인** 채택
  - mdast → n2ast → 실행 가능 코드 변환 체인
  - 플러그인 아키텍처로 확장성 확보

```
MDX 파이프라인:
.mdx → [parse] → mdast → [transform] → hast → [transform] → esast → [serialize] → .js

QLN 2.0 파이프라인 (제안):
.n2  → [parse] → n2ast → [validate] → typed-ast → [compile] → execution plan → [run] → result
```

### 1.3 mdschema — 선언적 MD 검증 ⭐⭐⭐

> YAML 스키마로 MD 구조를 검증하는 도구. 규칙 강제의 실현 가능성을 보여준다.

- **핵심**: YAML로 문서 구조 정의 → AST 파싱 → 규칙 위반 검출
- **검증 항목**: 헤딩, 코드 블록, 이미지, 테이블, 리스트, 링크, 프론트매터
- **시사점**:
  - `.n2` 스키마 검증에 유사 접근 가능
  - 그러나 mdschema는 문서 구조만 검증 — 행동 강제는 못 함
  - QLN 2.0은 **구조 검증 + 행동 강제** 모두 필요

---

## 2. AI 에이전트 거버넌스 연구

### 2.1 Agent Behavioral Contracts (ABC) ⭐⭐⭐⭐⭐

> **arxiv 논문** — Design-by-Contract 원칙을 AI 에이전트에 적용

- **핵심**: preconditions, invariants, governance policies, recovery mechanisms를 런타임에 강제
- **시사점**:
  - Soul 부팅 시퀀스 → **precondition**: `n2_boot`가 호출되지 않으면 다른 도구 사용 불가
  - 패키지 설치 금지 → **invariant**: `npm install` 패턴 감지 시 자동 차단
  - 에러 복구 → **recovery**: 빌드 실패 시 자동 롤백 메커니즘

```rust
// QLN 2.0 계약 예시 (Rust pseudocode)
#[contract]
struct BootSequence {
    #[precondition("project must be specified")]
    project: String,
    
    #[invariant("boot must complete before work")]
    boot_completed: bool,
    
    #[postcondition("handoff must be reported")]
    fn execute(&mut self) -> Result<BootReport, ContractViolation>;
}
```

### 2.2 AgentGuard — 런타임 검증 프레임워크 ⭐⭐⭐⭐

> **arxiv 논문** — Dynamic Probabilistic Assurance를 통한 지속적 검증

- **핵심**: 에이전트의 I/O를 관찰 → 이벤트 추상화 → MDP(Markov Decision Process) 모델 → 실시간 행동 검증
- **방법론**: 온라인 학습으로 에이전트 행동 패턴을 동적으로 모델링
- **시사점**:
  - QLN 2.0에 **행동 관찰 레이어** 추가: 도구 호출 패턴 모니터링
  - 이상 행동 감지: AI가 규칙을 일탈할 때 자동 경고/차단
  - Compliance Score 자동 산출

### 2.3 Open Agent Governance Specification (OAGS) ⭐⭐⭐

> 에이전트 거버넌스 표준 — 결정론적 ID, 정책, 감사

- **5대 원시 요소**: 결정론적 ID, 선언적 정책, 런타임 강제, 구조화된 감사, 암호화 검증
- **시사점**:
  - 에이전트별 고유 ID → Soul의 agent 시스템과 매핑
  - 선언적 정책 → `.n2` 파일로 정책 선언, Rust로 강제
  - 감사 로그 → Soul Ledger와 통합

### 2.4 MCP 2.0 (2024-12) ⭐⭐⭐⭐

> Model Context Protocol 2.0 — 구조화된 스키마 + 서버사이드 검증

- **핵심**: 인증 경계 (credential exposure 차단), 구조화된 스키마 + 서버사이드 검증 (injection 방지)
- **결정론적 + 테스트 가능한 AI 오퍼레이션** 목표
- **시사점**:
  - QLN 2.0의 Tool Layer를 MCP 2.0 스키마와 정렬
  - 서버사이드 검증 패턴을 Contract Layer에 반영

---

## 3. Rust 파서/컴파일러 생태계

### 3.1 파서 라이브러리 비교

| 라이브러리 | 유형 | 특징 | QLN 2.0 적합성 |
|-----------|------|------|---------------|
| **pest** | PEG 파서 생성기 | 별도 `.pest` 문법 파일, 선언적 | ⭐⭐⭐⭐⭐ 문법 정의에 최적 |
| **nom** | 파서 조합기 | 함수형, 바이너리/텍스트 모두 가능 | ⭐⭐⭐⭐ 저레벨 제어 필요 시 |
| **LALRPOP** | LR(1) 파서 생성기 | 전통적 파서 생성, 문법 중심 | ⭐⭐⭐ 복잡한 문법에 적합 |
| **parol** | LL(k)/LALR(1) | 읽기 쉬운 문법, 자동 AST 생성 | ⭐⭐⭐ |
| **chumsky** | 파서 조합기 | 에러 복구 특화, 친절한 에러 메시지 | ⭐⭐⭐⭐ UX 중시 시 적합 |

**추천**: Phase 2에서 `pest`로 시작 → 문법 안정화 후 성능 필요 시 `nom`으로 마이그레이션

### 3.2 Rust DSL 설계 패턴

1. **임베디드 DSL**: Rust 매크로(`macro_rules!`, `proc-macro`)로 Rust 내 DSL 구현
   - 장점: Rust 타입 시스템 활용 가능
   - 단점: 비개발자에게 진입장벽

2. **외부 DSL**: `.n2` 파일을 독립 언어로 설계 → Rust 파서로 해석
   - 장점: 자유로운 문법 설계, MD 친화적
   - 단점: 파서/컴파일러 전부 구현 필요

**추천**: **외부 DSL** — MD 가독성을 최우선으로 유지하되 Rust 컴파일러로 강제

---

## 4. SQL + 시맨틱 하이브리드 쿼리

### 4.1 NL2SQL 하이브리드 엔진 연구 동향

2024년 연구에서 활발한 분야:
- **LLM 기반 Text-to-SQL**: 자연어 → SQL 변환 (RAG + Graph RAG 활용)
- **벡터 임베딩**: 테이블/컬럼 메타데이터를 임베딩 → 시맨틱 매칭으로 정확도 향상
- **하이브리드 쿼리**: 복잡한 쿼리는 LLM, 단순 쿼리는 규칙 기반 → 비용 효율화

### 4.2 QLN 2.0 쿼리 엔진 설계

```sql
-- QLN 2.0 쿼리 예시 (제안)

-- 정확 매칭: SQL 레이어
SELECT tool FROM tools WHERE category = 'navigation' AND permission = 'allowed';

-- 시맨틱 매칭: Llama 레이어
SEMANTIC_SEARCH("사이트 이동하고 싶어") FROM tools LIMIT 5;

-- 하이브리드: SQL 필터 + 시맨틱 순위
SELECT * FROM tools 
WHERE category = 'browser' 
ORDER BY SEMANTIC_SIMILARITY("페이지 내용 읽기") DESC 
LIMIT 3;
```

### 4.3 벡터 저장소 전략

| 옵션 | 특징 | 추천 |
|------|------|------|
| SQLite + sqlite-vss | 경량, 단일 파일, 기존 Soul 인프라와 호환 | ⭐⭐⭐⭐⭐ |
| Qdrant | 전문 벡터 DB, 고성능 | ⭐⭐⭐ (별도 서비스 필요) |
| Milvus | 대규모, 분산 | ⭐⭐ (오버스펙) |
| In-memory (Rust) | 초저지연, 커스텀 | ⭐⭐⭐⭐ (소규모에 적합) |

**추천**: SQLite + sqlite-vss → Soul의 기존 SQLite 인프라와 자연스럽게 통합

---

## 5. QLN 2.0 레이어 상세 설계

### Layer 1: Transport Layer 🌐

```
역할: 통신 인프라
기존: HTTP REST + CDP (QLN 1.0)
개선: + WebSocket 양방향 + Unix Socket (로컬 최적화)
프로토콜: MessagePack 또는 CBOR (JSON보다 컴팩트)
```

### Layer 2: Tool Layer 🔌

```
역할: 도구 등록, 바인딩, 접근 제어
기존: 블랙리스트 기반 (SECURITY_PLUGINS Set)
개선: 
  - 화이트리스트 + 역할 기반 접근 제어 (RBAC)
  - 도구 스키마 검증 (입출력 타입 강제)
  - 도구 버전 관리
```

### Layer 3: Rule Layer 📜

```
역할: 규칙 파싱, 컴파일, 타입 검증
기존: MD 텍스트 (강제력 없음)
개선:
  - .n2 파일 → Rust 컴파일러 → 실행 가능 규칙
  - 타입 시스템: 도구 인자 타입 체크
  - 스키마: 필수 필드, 선택 필드, 포맷 검증
```

### Layer 4: Contract Layer 🔒

```
역할: 에이전트 행동 계약 강제
기존: 없음 (GEMINI.md에 텍스트로 기술)
개선:
  - Precondition: "부팅 전 작업 불가"
  - Postcondition: "세션 종료 시 n2_work_end 필수"
  - Invariant: "npm install 자동 실행 금지"
  - Recovery: 계약 위반 시 자동 복구/차단 메커니즘
```

### Layer 5: Query Layer 📊

```
역할: 구조화 데이터 쿼리, 상태 관리
기존: 없음 (JSON 직접 조작)
개선:
  - SQL 기반 쿼리 (도구/규칙/상태 테이블)
  - 트랜잭션: 워크플로우 단계 간 원자성 보장
  - 상태 머신: 에이전트 상태 전이 관리 (IDLE → BOOTED → WORKING → ENDING)
```

### Layer 6: Semantic Layer 🧠

```
역할: 의미 기반 이해, 검색, 매칭
기존: 없음 (이름 기반 매칭만)
개선:
  - Llama/nomic-embed-text 임베딩
  - 도구 시맨틱 검색: "사이트 이동" → ai_surf 자동 매칭
  - 규칙 의도 파악: 새 상황에서 기존 규칙 유추 적용
  - 컨텍스트 인식: 현재 워크플로우 맥락에 맞는 도구 추천
```

---

## 6. 핵심 논문 및 참고 자료

### 학술 논문
| 제목 | 출처 | 핵심 내용 |
|------|------|----------|
| Agent Behavioral Contracts (ABC) | arxiv (2024) | Design-by-Contract → AI 에이전트 런타임 강제 |
| AgentGuard: Runtime Verification of Agentic AI | arxiv (2024) | Dynamic Probabilistic Assurance, MDP 기반 행동 검증 |
| LLM-based Text-to-SQL Survey | arxiv (2024) | NL2SQL 아키텍처, 벤치마크, RAG 통합 |
| Formal Specification for AI Agent Runtimes | zylos.ai (2025) | TLA+ 기반 형식 검증, 신뢰 경계 |
| LLM Code Generation for Domain-Specific Languages | arxiv (2024-2025) | 저자원 언어/DSL LLM 코드 생성 |

### 오픈소스 프로젝트
| 프로젝트 | URL | 참고 포인트 |
|---------|-----|----------|
| Typst | github.com/typst/typst | Rust 마크업 컴파일러 아키텍처 |
| MDX | mdxjs.com | 다단계 AST 변환 파이프라인 |
| mdschema | github.com/mdschema | 선언적 MD 스키마 검증 |
| pest | pest.rs | PEG 파서 생성기 (Rust) |
| nom | github.com/rust-bakery/nom | 파서 조합기 (Rust) |
| LALRPOP | github.com/lalrpop/lalrpop | LR(1) 파서 생성기 (Rust) |
| chumsky | github.com/zesterer/chumsky | 에러 복구 특화 파서 (Rust) |

### 규격/표준
| 규격 | 핵심 내용 |
|------|----------|
| Open Agent Governance Specification (OAGS) | 에이전트 거버넌스 5대 원시 요소 |
| MCP 2.0 (2024-12) | 구조화 스키마 + 서버사이드 검증 |
| EU AI Act (2024-2026) | AI 리스크 관리, 투명성, 감사 의무 |
| NIST AI Agent Standards (2026-02) | 보안, ID, 상호운용성 |

---

## 7. 다음 단계 (Phase 2 준비)

1. **`.n2` 파일 문법 초안 설계** — Typst + MD 문법을 참고한 하이브리드 문법
2. **pest 기반 프로토타입 파서** — `.n2` → AST 변환 PoC
3. **SQLite + sqlite-vss 시맨틱 검색 PoC** — 도구 매칭 실험
4. **Soul 부팅 시퀀스 `.n2` 변환** — 실전 테스트 케이스
5. **Contract Layer 설계** — ABC 논문 기반 계약 시스템 상세 설계
