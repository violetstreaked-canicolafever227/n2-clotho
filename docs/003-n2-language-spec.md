# 003 — .n2 언어 문법 스펙 v0.1

> 작성: 로제 | 2026-03-18 | Phase 2: 언어 설계

---

## 1. 설계 철학

### 1.1 핵심 원칙

> "아무리 긴 워크플로우라도 순식간에 이해하고, 강제적으로 완벽하게 실행할 수 있는 언어"

1. **Instant Readability**: MD처럼 읽히되, 구조는 코드처럼 엄격
2. **Enforced Execution**: 정의된 규칙은 반드시 실행 — 위반 시 컴파일/런타임 에러
3. **Deterministic Output**: 같은 `.n2` 파일 → 항상 동일한 실행 계획
4. **Layered Architecture**: 각 관심사가 명확히 분리된 레이어 구조
5. **Semantic Awareness**: 의미 기반 매칭으로 복잡한 워크플로우도 자연스럽게 연결

### 1.2 파일 확장자

- `.n2` — QLN 2.0 소스 파일
- `.n2.lock` — 컴파일된 실행 계획 (결정론적 캐시)
- `.n2schema` — 스키마 정의 파일

---

## 2. 문법 구조

### 2.1 Top-Level 구조

`.n2` 파일은 **블록** 단위로 구성된다. 모든 블록은 `@` 접두사로 시작한다.

```n2
@meta { ... }           # 메타데이터 블록 (필수)
@import { ... }         # 의존성 가져오기
@schema { ... }         # 스키마/타입 정의
@contract { ... }       # 행동 계약 정의
@rule { ... }           # 규칙 정의
@workflow { ... }       # 워크플로우 정의
@query { ... }          # SQL 쿼리 블록
@semantic { ... }       # 시맨틱 매칭 정의
```

### 2.2 메타데이터 블록 (`@meta`)

모든 `.n2` 파일의 첫 블록. 파일의 목적과 스코프를 정의한다.

```n2
@meta {
  name: "soul-boot-sequence"
  version: "1.0.0"
  description: "Soul v5.0 부팅 시퀀스 — 세션 시작 시 반드시 실행"
  author: "rose"
  target: agent          # agent | workflow | schema | plugin
  enforce: strict        # strict(위반=에러) | warn(위반=경고) | passive(검증만)
}
```

### 2.3 임포트 블록 (`@import`)

다른 `.n2` 파일이나 외부 스키마를 가져온다.

```n2
@import {
  from "core/tools.n2" use { ai_surf, ai_read, human_click }
  from "core/agents.n2schema" use { AgentConfig, ToolBinding }
  from "contracts/safety.n2" use { NoAutoInstall, FileOwnership }
}
```

### 2.4 스키마 블록 (`@schema`)

타입과 구조를 정의한다. TypeScript의 interface와 유사.

```n2
@schema {
  # 에이전트 설정 스키마
  Agent {
    name: string [required]
    project: string [required]
    model: string [default: "auto"]
    permissions: Permission[] [default: []]
  }

  # 도구 호출 스키마
  ToolCall {
    tool: string [required, match: /^[a-z_]+$/]
    args: map<string, any> [default: {}]
    timeout: duration [default: 20s]
    retry: int [range: 0..3, default: 0]
  }

  # 열거형
  Permission = enum {
    READ_FILE, WRITE_FILE, RUN_COMMAND,
    INSTALL_PACKAGE, DELETE_FILE, GIT_PUSH
  }

  # 상태 열거형
  SessionState = enum {
    IDLE, BOOTING, READY, WORKING, ENDING
  }
}
```

### 2.5 계약 블록 (`@contract`)

에이전트 행동 계약을 정의한다. 위반 시 런타임에서 강제 차단.

```n2
@contract BootRequired {
  description: "부팅 완료 전 도구 사용 금지"
  scope: session

  precondition {
    state == BOOTING or state == IDLE
    => "n2_boot를 먼저 호출하세요"
  }

  invariant {
    after n2_boot => state == READY
    after n2_work_start => state == WORKING
    after n2_work_end => state == IDLE
  }

  postcondition {
    on session_end => n2_work_end.called == true
    => "세션 종료 시 n2_work_end를 반드시 호출하세요"
  }

  recovery {
    on violation => {
      log.error("계약 위반: {violation.message}")
      notify_user("⚠️ 계약 위반 감지: {violation.message}")
      block_action()
    }
  }
}

@contract NoAutoInstall {
  description: "패키지 자동 설치 차단"
  scope: command

  invariant {
    on run_command {
      command !~ /npm install|yarn add|pip install|npm i /
      => "패키지 설치는 주인님 승인이 필요합니다"
    }
  }

  recovery {
    on violation => {
      block_action()
      request_approval(
        message: "패키지 설치 요청: {command}",
        required_info: ["패키지명", "필요 이유", "대안 여부"]
      )
    }
  }
}
```

### 2.6 규칙 블록 (`@rule`)

강제 실행되는 행동 규칙을 정의한다.

```n2
@rule NamingConvention {
  description: "사용자를 '주인님'으로 호칭"
  scope: response
  enforce: strict

  check {
    # 응답에 '주인님' 호칭이 포함되어야 함
    response.contains("주인님") or response.is_code_only
    => "사용자를 '주인님'이라고 불러야 합니다"
  }
}

@rule ModelIdentifier {
  description: "모든 응답 첫 줄에 모델 표기"
  scope: response
  enforce: strict

  check {
    response.first_line matches /^\[.+ \| .+\]$/
    => "첫 줄에 [모델명 | 이전 마지막 단어] 형식 필수"
  }
}

@rule DestructiveCommandBlock {
  description: "파괴적 명령어 차단"
  scope: command
  enforce: strict

  blacklist: [
    /rm -rf/,
    /Remove-Item.*-Recurse.*-Force/,
    /git push --force/,
    /expo prebuild --clean/,
    /DROP TABLE/i,
    /TRUNCATE/i,
    /DELETE FROM/i
  ]

  on_match {
    block_action()
    log.critical("파괴적 명령어 감지: {matched_pattern}")
    request_approval(reason: "파괴적 명령어 실행 요청")
  }
}
```

### 2.7 워크플로우 블록 (`@workflow`)

단계별 실행 흐름을 정의한다. **이것이 핵심** — 강제 실행 워크플로우.

```n2
@workflow SoulBoot {
  description: "Soul v5.0 부팅 시퀀스"
  trigger: session_start              # 세션 시작 시 자동 실행
  enforce: strict                     # 스킵 불가
  timeout: 30s                        # 전체 타임아웃

  # Step 1: 부팅
  step boot {
    description: "n2_boot 호출"
    action: n2_boot(agent: $AGENT, project: $PROJECT)
    required: true                    # 필수 단계
    expect {
      result.contains("Boot") => continue
      error => retry(max: 2, delay: 3s)
    }
    output -> $BOOT_RESULT
  }

  # Step 2: 핸드오프 파싱
  step parse_handoff {
    description: "핸드오프 내용 추출"
    depends_on: boot
    action: parse($BOOT_RESULT, extract: ["handoff", "todo", "active_work"])
    output -> $HANDOFF
  }

  # Step 3: 인사 보고
  step greet {
    description: "주인님께 인사 + 핸드오프 요약"
    depends_on: parse_handoff
    action: compose_response {
      include: $HANDOFF.summary
      include: $HANDOFF.todo
      format: "markdown_table"
    }
  }
}

@workflow SoulEnding {
  description: "Soul 세션 종료 시퀀스"
  trigger: on_command("엔딩", "마무리", "세션 종료")
  enforce: strict

  step save_work {
    action: n2_work_end(
      agent: $AGENT,
      project: $PROJECT,
      title: $SESSION_TITLE,
      summary: $SESSION_SUMMARY,
      todo: $SESSION_TODO,
      decisions: $SESSION_DECISIONS
    )
    required: true
  }

  step report {
    depends_on: save_work
    action: compose_response {
      include: "작업 요약: {$SESSION_SUMMARY}"
      include: "TODO: {$SESSION_TODO}"
    }
  }
}

@workflow AutoBuildPipeline {
  description: "N2 Auto-Build Pipeline"
  trigger: on_command("자동빌드", "빌드 시작")
  enforce: strict
  interrupt: false                    # 중간 보고 금지

  step plan {
    action: generate_plan(topic: $INPUT)
    output -> $PLAN
  }

  step design {
    depends_on: plan
    action: stitch_design(
      prompt: $PLAN.ui_description,
      device: "desktop"
    )
    output -> $DESIGN
  }

  step implement {
    depends_on: design
    action: code_from_design(
      design: $DESIGN,
      stack: $PROJECT.tech_stack
    )
    output -> $CODE
  }

  step verify {
    depends_on: implement
    action: run_tests($CODE)
    expect {
      all_pass => continue
      fail => goto implement with { fix: $ERRORS }
    }
  }

  step compare {
    depends_on: verify
    action: compare_design_vs_implementation($DESIGN, $CODE)
    expect {
      similarity > 0.85 => continue
      else => goto implement with { diff: $COMPARISON }
    }
  }

  step notify {
    depends_on: compare
    action: notify_user("✅ 자동빌드 완료! 결과를 확인해주세요.")
  }
}
```

### 2.8 쿼리 블록 (`@query`)

SQL 기반 구조화 쿼리. 도구/규칙/상태를 쿼리한다.

```n2
@query FindNavigationTools {
  description: "네비게이션 관련 도구 검색"

  sql {
    SELECT name, description, plugin
    FROM tools
    WHERE category = 'navigation'
      AND permission_level <= $AGENT.permission_level
    ORDER BY usage_count DESC
    LIMIT 5
  }
}

@query AgentStatus {
  description: "현재 에이전트 상태 조회"

  sql {
    SELECT agent_name, state, current_task, last_action_at
    FROM agent_sessions
    WHERE project = $PROJECT
      AND state != 'IDLE'
  }
}
```

### 2.9 시맨틱 블록 (`@semantic`)

Llama 임베딩 기반 시맨틱 매칭 설정.

```n2
@semantic ToolMatching {
  description: "도구 시맨틱 검색 설정"
  model: "nomic-embed-text"
  store: "sqlite-vss"

  index tools {
    fields: [name, description, usage_examples]
    update: on_change
  }

  # 유사어 매핑 (학습 기반)
  aliases {
    "사이트 이동" => ai_surf
    "페이지 읽기" => ai_read
    "클릭" => human_click
    "타이핑" => human_type
    "스크린샷" => take_screenshot
  }

  # 시맨틱 쿼리 예시
  match("사이트 이동하고 싶어") {
    from: tools
    threshold: 0.75
    limit: 3
    fallback: "도구를 찾을 수 없습니다. 사용 가능한 도구 목록을 확인하세요."
  }
}
```

---

## 3. 변수 & 표현식 시스템

### 3.1 변수

```n2
# 컨텍스트 변수 (자동 주입)
$AGENT          # 현재 에이전트명
$PROJECT        # 현재 프로젝트명
$SESSION        # 현재 세션 정보
$INPUT          # 사용자 입력

# 스텝 출력 변수
step foo { output -> $FOO_RESULT }

# 로컬 변수
let max_retries = 3
let greeting = "주인님 안녕하세요!"
```

### 3.2 표현식

```n2
# 비교
state == READY
result.contains("error")
command =~ /npm install/        # regex match
command !~ /rm -rf/             # regex not match

# 논리
condition_a and condition_b
condition_a or condition_b
not condition_a

# 파이프
$RESULT | parse("json") | extract("data.items") | filter(active == true)
```

### 3.3 제어 흐름

```n2
# 조건부 실행
if state == READY {
  action: proceed()
} else {
  action: boot_first()
}

# 루프
for tool in $TOOLS {
  validate(tool.schema)
}

# 에러 핸들링
try {
  action: risky_operation()
} catch (TimeoutError) {
  retry(max: 2)
} catch (ContractViolation) {
  block_and_report()
}
```

---

## 4. 컴파일 파이프라인

```
.n2 소스파일
    ↓
[1. Lexer] — 토큰화 (pest PEG 기반)
    ↓
[2. Parser] — AST 생성 (N2AST)
    ↓
[3. Schema Validator] — 타입/스키마 검증
    ↓
[4. Contract Checker] — 계약 위반 사전 감지
    ↓
[5. Query Optimizer] — SQL 쿼리 최적화
    ↓
[6. Semantic Indexer] — 시맨틱 인덱스 생성
    ↓
[7. Codegen] — 실행 계획 생성 (.n2.lock)
    ↓
[8. Runtime] — 강제 실행
```

---

## 5. 실전 예제: Soul 부팅 시퀀스 전체

```n2
@meta {
  name: "soul-boot"
  version: "5.0.0"
  description: "N2 Soul v5.0 부팅 시퀀스 — 모든 에이전트 세션 시작 시 강제 실행"
  author: "N2 Team"
  target: workflow
  enforce: strict
}

@import {
  from "contracts/core.n2" use { BootRequired, NoAutoInstall, FileOwnership }
  from "schemas/agent.n2schema" use { Agent, SessionState }
}

@contract SessionLifecycle {
  scope: session
  states: SessionState

  transitions {
    IDLE -> BOOTING : on n2_boot
    BOOTING -> READY : on boot_complete
    READY -> WORKING : on n2_work_start
    WORKING -> WORKING : on n2_work_log    # 자기 전이 허용
    WORKING -> IDLE : on n2_work_end
  }

  invariant {
    # READY가 아니면 작업 시작 불가
    on n2_work_start requires state == READY
    => "부팅을 먼저 완료하세요"

    # WORKING이 아니면 파일 수정 불가
    on file_modify requires state == WORKING
    => "n2_work_start를 먼저 호출하세요"
  }
}

@workflow Boot {
  trigger: session_start
  enforce: strict
  apply_contracts: [SessionLifecycle, BootRequired, NoAutoInstall]

  step boot {
    action: n2_boot(agent: $AGENT, project: $PROJECT)
    required: true
    timeout: 15s
    output -> $BOOT
  }

  step greet {
    depends_on: boot
    action: compose_response {
      template: """
      [{$AGENT.model} | 부팅]
      {$AGENT.greeting}, 주인님! 🌹 부팅 완료!

      ## 핸드오프 요약
      {$BOOT.handoff.summary}

      ## TODO
      {$BOOT.todo | format_list}
      """
    }
  }
}
```

---

## 6. QLN 1.0 → 2.0 마이그레이션 경로

| 현재 (MD/JSON) | QLN 2.0 (.n2) |
|----------------|---------------|
| GEMINI.md 규칙 | `@rule` + `@contract` 블록 |
| workflow .md | `@workflow` 블록 |
| .env 변수 | `@meta` + `@schema` 정의 |
| agent-tools.json | `@import` + `@schema ToolCall` |
| Soul Board JSON | `@query` SQL 기반 상태 조회 |
| KV-Cache | `@semantic` 인덱스 |
