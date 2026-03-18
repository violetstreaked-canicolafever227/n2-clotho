// Contract Checker — @contract 상태머신 시뮬레이션 + 위반 사전 감지
use crate::ast::*;
use std::collections::{HashMap, HashSet};

/// 상태머신 정의
#[derive(Debug)]
pub struct StateMachine {
    pub name: String,
    pub states: Vec<String>,
    pub initial_state: String,
    pub transitions: Vec<Transition>,
    pub invariants: Vec<String>,
}

#[derive(Debug)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub on_event: String,
}

/// 상태머신 런타임 (시뮬레이션용)
#[derive(Debug)]
pub struct ContractRuntime {
    pub machines: HashMap<String, StateMachine>,
    pub current_states: HashMap<String, String>,
}

/// 계약 체크 결과
#[derive(Debug)]
pub struct ContractViolation {
    pub contract: String,
    pub violation_type: ViolationType,
    pub message: String,
}

#[derive(Debug)]
pub enum ViolationType {
    InvalidTransition,
    MissingState,
    UnreachableState,
    DeadlockDetected,
}

impl std::fmt::Display for ContractViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self.violation_type {
            ViolationType::InvalidTransition => "🚫",
            ViolationType::MissingState => "❓",
            ViolationType::UnreachableState => "🔒",
            ViolationType::DeadlockDetected => "💀",
        };
        write!(f, "{} [{}] {:?}: {}", icon, self.contract, self.violation_type, self.message)
    }
}

impl ContractRuntime {
    /// N2File에서 @contract 블록들을 추출하여 상태머신들을 구성
    pub fn from_file(file: &N2File) -> Self {
        let mut machines = HashMap::new();

        for block in &file.blocks {
            if let Block::Contract(ct) = block {
                if ct.states.is_some() && !ct.transitions.is_empty() {
                    let sm = StateMachine {
                        name: ct.name.clone(),
                        states: Vec::new(), // 전이에서 자동 추출
                        initial_state: ct.transitions.first()
                            .map(|t| t.from.clone())
                            .unwrap_or_default(),
                        transitions: ct.transitions.iter().map(|t| Transition {
                            from: t.from.clone(),
                            to: t.to.clone(),
                            on_event: t.on_event.clone(),
                        }).collect(),
                        invariants: ct.invariants.clone(),
                    };
                    machines.insert(ct.name.clone(), sm);
                }
            }
        }

        // 각 머신의 states를 전이에서 자동 추출
        for machine in machines.values_mut() {
            let mut states = HashSet::new();
            for t in &machine.transitions {
                states.insert(t.from.clone());
                states.insert(t.to.clone());
            }
            machine.states = states.into_iter().collect();
        }

        // 초기 상태 설정
        let mut current_states = HashMap::new();
        for (name, machine) in &machines {
            current_states.insert(name.clone(), machine.initial_state.clone());
        }

        ContractRuntime { machines, current_states }
    }

    /// 정적 분석: 상태머신 무결성 검사
    pub fn check_integrity(&self) -> Vec<ContractViolation> {
        let mut violations = Vec::new();

        for (name, machine) in &self.machines {
            // 1. 도달 불가능한 상태 감지
            let reachable = self.find_reachable_states(machine);
            for state in &machine.states {
                if !reachable.contains(state) && *state != machine.initial_state {
                    violations.push(ContractViolation {
                        contract: name.clone(),
                        violation_type: ViolationType::UnreachableState,
                        message: format!("상태 '{}'에 도달할 수 있는 전이가 없습니다", state),
                    });
                }
            }

            // 2. 데드락 감지 (전이가 하나도 없는 비-종료 상태)
            for state in &machine.states {
                let has_outgoing = machine.transitions.iter().any(|t| t.from == *state);
                let is_likely_terminal = state.to_lowercase().contains("idle")
                    || state.to_lowercase().contains("end")
                    || state.to_lowercase().contains("done");

                if !has_outgoing && !is_likely_terminal {
                    violations.push(ContractViolation {
                        contract: name.clone(),
                        violation_type: ViolationType::DeadlockDetected,
                        message: format!("상태 '{}'에서 나가는 전이가 없습니다 (데드락 가능)", state),
                    });
                }
            }

            // 3. 존재하지 않는 상태로의 전이 감지
            for t in &machine.transitions {
                if !machine.states.contains(&t.from) {
                    violations.push(ContractViolation {
                        contract: name.clone(),
                        violation_type: ViolationType::MissingState,
                        message: format!("전이 출발 상태 '{}'가 정의되지 않았습니다", t.from),
                    });
                }
                if !machine.states.contains(&t.to) {
                    violations.push(ContractViolation {
                        contract: name.clone(),
                        violation_type: ViolationType::MissingState,
                        message: format!("전이 도착 상태 '{}'가 정의되지 않았습니다", t.to),
                    });
                }
            }
        }

        violations
    }

    /// 이벤트 시뮬레이션: 전이가 유효한지 확인
    pub fn simulate_event(&mut self, contract_name: &str, event: &str) -> Result<String, ContractViolation> {
        let current = self.current_states.get(contract_name)
            .ok_or_else(|| ContractViolation {
                contract: contract_name.to_string(),
                violation_type: ViolationType::MissingState,
                message: format!("계약 '{}'를 찾을 수 없습니다", contract_name),
            })?
            .clone();

        let machine = self.machines.get(contract_name)
            .ok_or_else(|| ContractViolation {
                contract: contract_name.to_string(),
                violation_type: ViolationType::MissingState,
                message: format!("상태머신 '{}'을 찾을 수 없습니다", contract_name),
            })?;

        // 유효한 전이 찾기
        let valid_transition = machine.transitions.iter()
            .find(|t| t.from == current && t.on_event == event);

        match valid_transition {
            Some(t) => {
                let new_state = t.to.clone();
                self.current_states.insert(contract_name.to_string(), new_state.clone());
                Ok(new_state)
            }
            None => {
                let valid_events: Vec<_> = machine.transitions.iter()
                    .filter(|t| t.from == current)
                    .map(|t| t.on_event.as_str())
                    .collect();
                Err(ContractViolation {
                    contract: contract_name.to_string(),
                    violation_type: ViolationType::InvalidTransition,
                    message: format!(
                        "상태 '{}'에서 이벤트 '{}'는 유효하지 않습니다. 가능한 이벤트: [{}]",
                        current, event, valid_events.join(", ")
                    ),
                })
            }
        }
    }

    /// BFS로 초기 상태에서 도달 가능한 상태 찾기
    fn find_reachable_states(&self, machine: &StateMachine) -> HashSet<String> {
        let mut reachable = HashSet::new();
        let mut queue = vec![machine.initial_state.clone()];
        reachable.insert(machine.initial_state.clone());

        while let Some(state) = queue.pop() {
            for t in &machine.transitions {
                if t.from == state && !reachable.contains(&t.to) {
                    reachable.insert(t.to.clone());
                    queue.push(t.to.clone());
                }
            }
        }

        reachable
    }

    /// 상태머신 요약 출력
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        for (name, machine) in &self.machines {
            let current = self.current_states.get(name).unwrap_or(&"?".to_string()).clone();
            lines.push(format!("📋 {} | states: {} | current: {} | transitions: {}",
                name,
                machine.states.len(),
                current,
                machine.transitions.len(),
            ));
            for t in &machine.transitions {
                lines.push(format!("   {} -[{}]-> {}", t.from, t.on_event, t.to));
            }
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_n2;

    #[test]
    fn test_state_machine_simulation() {
        let source = r#"
@meta {
  name: "test-contract"
  enforce: strict
}

@contract SessionLifecycle {
  scope: session
  states: SessionState

  transitions {
    IDLE -> BOOTING : on n2_boot
    BOOTING -> READY : on boot_complete
    READY -> WORKING : on n2_work_start
    WORKING -> IDLE : on n2_work_end
  }
}
"#;
        let file = parse_n2(source).unwrap();

        // 디버그: contract 블록의 transitions 확인
        for block in &file.blocks {
            if let crate::ast::Block::Contract(ct) = block {
                eprintln!("Contract: {}, states: {:?}, transitions: {}", ct.name, ct.states, ct.transitions.len());
                for t in &ct.transitions {
                    eprintln!("  {} -> {} : on {}", t.from, t.to, t.on_event);
                }
            }
        }

        let mut runtime = ContractRuntime::from_file(&file);

        eprintln!("Machines: {:?}", runtime.machines.keys().collect::<Vec<_>>());
        eprintln!("Current states: {:?}", runtime.current_states);

        // 머신이 존재하는지 먼저 확인
        assert!(!runtime.machines.is_empty(), "상태머신이 비어있습니다");
        assert!(runtime.machines.contains_key("SessionLifecycle"), "SessionLifecycle 머신이 없습니다");

        // 초기 상태: IDLE
        let initial = runtime.current_states.get("SessionLifecycle").cloned();
        assert_eq!(initial, Some("IDLE".to_string()), "초기 상태가 IDLE이어야 합니다");

        // IDLE -> BOOTING (n2_boot)
        let result = runtime.simulate_event("SessionLifecycle", "n2_boot");
        assert!(result.is_ok(), "전이 실패: {:?}", result.err());
        assert_eq!(result.unwrap(), "BOOTING");

        // BOOTING -> READY (boot_complete)
        let result = runtime.simulate_event("SessionLifecycle", "boot_complete");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "READY");

        // READY에서 n2_boot는 불가
        let result = runtime.simulate_event("SessionLifecycle", "n2_boot");
        assert!(result.is_err());
    }

    #[test]
    fn test_integrity_check() {
        let source = r#"
@meta {
  name: "test"
  enforce: strict
}

@contract Test {
  states: S

  transitions {
    A -> B : on go
    B -> C : on next
    C -> A : on reset
  }
}
"#;
        let file = parse_n2(source).unwrap();
        let runtime = ContractRuntime::from_file(&file);
        let violations = runtime.check_integrity();
        // A->B->C->A 순환이므로 도달 불가/데드락 없음
        assert!(violations.is_empty(), "위반 감지됨: {:?}", violations);
    }
}
