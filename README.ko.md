EN [English](README.md)

# 🧵 Clotho — AI 에이전트의 운명의 실

[![npm version](https://img.shields.io/npm/v/n2-clotho.svg)](https://www.npmjs.com/package/n2-clotho)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Rust](https://img.shields.io/badge/Built_with-Rust-dea584?logo=rust)](https://www.rust-lang.org/)
[![WASM](https://img.shields.io/badge/Runs_on-WASM-654ff0?logo=webassembly)](https://webassembly.org/)
[![GitHub stars](https://img.shields.io/github/stars/choihyunsus/n2-clotho?style=social)](https://github.com/choihyunsus/n2-clotho)

> **마크다운 규칙은 죽었다. `.n2` 만세.**

Clotho는 AI 에이전트를 위한 **컴파일형 명령어 언어**입니다. 깨지기 쉬운 마크다운 기반 규칙(GEMINI.md, .cursorrules, CLAUDE.md)을 **강제 실행 가능하고, 타입 체크되고, 결정론적인** 사양으로 대체합니다.

[클로토(Clotho)](https://ko.wikipedia.org/wiki/%ED%81%B4%EB%A1%9C%ED%86%A0)는 운명의 실을 잣는 그리스 여신입니다 — 규칙을 한 번 정의하면, 그것이 곧 **운명**이 되니까요.

```
# Before: GEMINI.md (정중한 부탁)
"rm -rf는 실행하지 말아주세요. 감사합니다!"
→ AI: "네!" *rm -rf 실행*

# After: rules.n2 (컴파일된 법률)
@rule NoDestructive {
  blacklist: [/rm -rf/, /DROP TABLE/i]
}
→ AI가 rm -rf 시도 → ❌ 차단. 예외 없음.
```

## 🎯 문제

### 🧵 Clotho를 4컷 만화로

<p align="center">
  <img src="docs/images/clotho-comic.png" alt="Clotho 4컷 만화" width="700" />
</p>

> **1화**: AI가 마크다운 규칙을 읽는다... **2화**: ...그리고 무시한다.
> **3화**: Clotho가 `.n2` 컴파일된 법과 함께 등장. **4화**: 규칙 강제 실행. 예외 없음.

모든 AI 코딩 도구에는 각자의 마크다운 기반 규칙 시스템이 있습니다:

| 도구 | 규칙 파일 | 강제성 |
|------|-----------|:------:|
| Gemini | `GEMINI.md` | 🙏 희망 |
| Cursor | `.cursorrules` | 🙏 희망 |
| Claude | `CLAUDE.md` | 🙏 희망 |
| Windsurf | `.windsurfrules` | 🙏 희망 |
| System Prompt | `system.txt` | 🙏 희망 |

결과? **같은 규칙, 매번 다른 결과.** 에이전트가 규칙을 "읽기는" 하지만 일관성 없이 따릅니다. 컴파일도, 검증도, 강제 실행도 없습니다 — 그냥 바이브.

## 💡 솔루션

모든 것을 대체할 단 하나의 언어:

```
Before (.md + .json + 스킬 파일들):       After (.n2):
┌─────────────────────────────────┐    ┌──────────────────────┐
│ GEMINI.md      → 행동 규칙      │    │                      │
│ .cursorrules   → 에디터 규칙    │    │   project.n2         │
│ workflows/*.md → 스킬 단계      │    │                      │
│ config.json    → MCP 설정      │    │   파일 하나.          │
│ system.txt     → 시스템 프롬프트 │    │   컴파일됨.           │
│ .env           → 변수          │    │   강제 실행.           │
└─────────────────────────────────┘    └──────────────────────┘
   6개+ 파일, 강제성 제로             1개 파일, 완전한 강제 실행
```

Clotho가 도입하는 `.n2` — 컴파일형 명령어 언어:

| 기능 | 설명 |
|------|------|
| **컴파일** | PEG 파서 → AST → 검증된 실행 계획 |
| **타입 안전성** | 스키마 정의 타입 + 제약조건 (`[required]`, `[range: 0..3]`) |
| **계약** | 상태머신 행동 계약 — 위반 = 행동 차단 |
| **강제 실행** | 블록별 `strict` / `warn` / `passive` 모드 |
| **결정론** | 같은 `.n2` 파일 → 항상 같은 실행 계획 |
| **SQL 쿼리** | 규칙을 DB처럼 조회: `SELECT * FROM rules WHERE scope = 'command'` |
| **시맨틱 매칭** | Ollama 기반 의도 → 도구 매핑 (선택사항) |

## ⚡ 빠른 시작

### 설치

> 💡 **가장 쉬운 방법?** AI에게 `.n2` 파일을 만들어달라고 부탁하세요.

```bash
# npm (WASM — Node.js에서 사용)
npm install n2-clotho
```

```javascript
// Node.js에서 사용
const { parse_n2_wasm, validate_n2_wasm, query_n2_wasm } = require('n2-clotho');

const ast = parse_n2_wasm(n2Source);        // 파싱 → AST JSON
const result = validate_n2_wasm(n2Source);  // 검증 → 에러/경고
const table = query_n2_wasm(n2Source, 'SELECT * FROM rules');  // SQL 쿼리
```

```bash
# 소스에서 빌드 (Rust 필요 — 전체 CLI)
git clone https://github.com/choihyunsus/n2-clotho.git
cd n2-clotho/compiler
cargo build --release

# 바이너리 위치: target/release/n2-compiler
```

### 첫 번째 `.n2` 파일 작성하기

```n2
# my-rules.n2 — 첫 번째 Clotho 규칙 세트

@meta {
  name: "my-project-rules"
  version: "1.0.0"
  description: "AI 에이전트 행동 규칙"
  enforce: strict
}

@rule NoAutoInstall {
  description: "무단 패키지 설치 차단"
  scope: command
  enforce: strict

  blacklist: [
    /npm install/,
    /yarn add/,
    /pip install/
  ]
}

@workflow Boot {
  description: "세션 시작 시퀀스"
  trigger: session_start
  enforce: strict

  step initialize {
    description: "프로젝트 컨텍스트 로드"
    action: load_context()
    required: true
    timeout: 15s
  }

  step greet {
    depends_on: initialize
    action: compose_response()
  }
}
```

### 컴파일 & 검증

```bash
# 파싱 및 AST 출력
n2c my-rules.n2

# 전체 검증 파이프라인: 파싱 → 스키마 체크 → 계약 검증
n2c validate my-rules.n2

# 상태머신 계약 시뮬레이션
n2c simulate my-rules.n2

# SQL로 규칙 쿼리
n2c query my-rules.n2 "SELECT * FROM rules WHERE enforce = 'strict'"
```

## 📐 언어 사양

### 8가지 블록 타입

모든 `.n2` 파일은 `@` 접두사 블록으로 구성됩니다:

```n2
@meta { ... }           # 파일 메타데이터 (필수)
@import { ... }         # 다른 .n2 파일 임포트
@schema { ... }         # 타입 & 스키마 정의
@contract { ... }       # 행동 계약 (상태머신)
@rule { ... }           # 강제 규칙 (블랙리스트, 체크)
@workflow { ... }       # 강제 단계별 워크플로우
@query { ... }          # SQL 기반 규칙 쿼리
@semantic { ... }       # 시맨틱 매칭 (Ollama 연동)
```

### @contract — 상태머신 행동 계약

가장 강력한 블록. **위반 불가능한** 상태 전이를 정의합니다:

```n2
@contract SessionLifecycle {
  scope: session
  states: SessionState

  transitions {
    IDLE -> BOOTING : on boot
    BOOTING -> READY : on boot_complete
    READY -> WORKING : on work_start
    WORKING -> WORKING : on work_log      # 자기 전이 허용
    WORKING -> IDLE : on work_end
  }

  invariant {
    on work_start requires state == READY
    => "작업 시작 전 부팅을 완료해야 합니다"

    on file_modify requires state == WORKING
    => "파일 수정 전 work_start를 호출해야 합니다"
  }
}
```

### @rule — 강제 규칙

실시간으로 행동을 차단하는 체크와 블랙리스트 정의:

```n2
@rule DestructiveCommandBlock {
  description: "파괴적 명령어 차단"
  scope: command
  enforce: strict

  blacklist: [
    /rm -rf/,
    /git push --force/,
    /DROP TABLE/i,
    /TRUNCATE/i,
    /expo prebuild --clean/
  ]
}
```

## 🏗️ 컴파일러 파이프라인

```
.n2 소스 파일
    ↓
[1. 렉서]              PEG 토큰화 (pest)
    ↓
[2. 파서]              AST 생성 (N2File → Blocks)
    ↓
[3. 스키마 검증기]      타입 & 제약 검사
    ↓
[4. 계약 체커]          상태머신 무결성 검증
    ↓
[5. 쿼리 옵티마이저]    SQL 쿼리 검증
    ↓
[6. 코드젠]            실행 계획 (.n2.lock)
    ↓
[7. 런타임]            강제 실행 ← Soul/QLN 필요
```

**1–6단계**는 Clotho(이 도구)가 처리합니다.
**7단계**는 [N2 Soul](https://github.com/choihyunsus/soul) 런타임이 처리합니다.

## 🕸️ N2 생태계 연동

Clotho는 N2 생태계의 일부입니다 — AI 에이전트 개발을 위한 MCP 네이티브 도구 모음:

| 패키지 | 역할 | npm |
|--------|------|-----|
| **Clotho** | 규칙 컴파일러 & 검증기 | `n2-clotho` |
| **Soul** | 에이전트 메모리 & 런타임 강제 실행 | `n2-soul` |
| **Arachne** | 코드 컨텍스트 어셈블리 (BM25 + 시맨틱) | `n2-arachne` |
| **Ark** | 보안 게이트 & 감사 | `n2-ark` |
| **QLN** | 도구 오케스트레이션 & 라우팅 | `n2-qln` |

### 단독 사용 vs 통합 사용

| 모드 | 기능 |
|------|------|
| **단독** (Clotho만) | `.n2` 파일 파싱, 검증, 시뮬레이션, 쿼리 |
| **+ Soul** | 런타임 강제 실행 — 계약이 실시간으로 위반을 차단 |
| **+ Arachne** | 코드 인식 규칙 — 계약이 실제 코드베이스 컨텍스트 참조 |
| **+ 풀스택** | 메모리, 보안, 도구를 포함한 완전한 AI 에이전트 거버넌스 |

## 🆚 마크다운 규칙 vs `.n2`

| 측면 | 마크다운 (GEMINI.md) | Clotho (.n2) |
|------|---------------------|-------------|
| **형식** | 자유형 텍스트 | 구조화된 블록 |
| **파싱** | 최선을 다해 | PEG 문법 → AST |
| **검증** | 없음 | 스키마 + 계약 + SQL |
| **강제 실행** | AI 재량 🙏 | 컴파일러 강제 ❌ |
| **상태 추적** | 없음 | 상태머신 계약 |
| **쿼리** | Ctrl+F | SQL 쿼리 |
| **결정론** | ❌ 매번 다름 | ✅ 같은 입력 → 같은 계획 |
| **에이전트 간 공유** | 복사-붙여넣기 | `@import`로 공유 `.n2` 파일 |
| **디버깅** | 문서 전체 읽기 | `n2c validate`로 오류 정확히 찾기 |

## 🛡️ 기술 스택

- **Rust** — 제로 코스트 추상화, 메모리 안전성
- **pest** — `.n2` 문법을 위한 PEG 파서 생성기
- **serde** — AST ↔ JSON 직렬화
- **WASM** — 브라우저/Node.js 컴파일 타겟 (선택)

## 📄 라이선스

Apache-2.0 — 자유롭게 사용, 수정, 배포할 수 있습니다.

## 🔗 링크

- [**npm: n2-clotho**](https://www.npmjs.com/package/n2-clotho) — Node.js용 WASM 바인딩
- [N2 Soul](https://github.com/choihyunsus/soul) — 에이전트 메모리 & 런타임
- [N2 Arachne](https://github.com/choihyunsus/n2-arachne) — 코드 컨텍스트 어셈블리
- [N2 QLN](https://github.com/choihyunsus/n2-qln) — 도구 오케스트레이션 & 라우팅
- [N2 Ark](https://github.com/choihyunsus/n2-ark) — 보안 검증

---

## ⭐ Star History

Clotho가 도움이 되셨다면, 스타를 눌러주세요! ⭐

[![Star History Chart](https://api.star-history.com/svg?repos=choihyunsus/n2-clotho&type=Date)](https://star-history.com/#choihyunsus/n2-clotho&Date)

---

> *"마크다운 규칙은 제안이다. Clotho 규칙은 법이다."*
