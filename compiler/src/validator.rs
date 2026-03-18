// Schema Validator — @schema 블록의 타입/필수필드/범위 검증
use crate::ast::*;
use std::collections::HashMap;

/// 검증 에러
#[derive(Debug)]
pub struct ValidationError {
    pub block: String,
    pub field: String,
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self.severity {
            Severity::Error => "❌",
            Severity::Warning => "⚠️",
        };
        write!(f, "{} [{}] {}: {}", icon, self.block, self.field, self.message)
    }
}

/// N2File 전체를 검증
pub fn validate(file: &N2File) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // 1. @meta 블록 필수 검증
    validate_meta_required(file, &mut errors);

    // 2. @schema 정의 수집
    let _schema_defs = collect_schema_defs(file);

    // 3. @workflow 검증
    validate_workflows(file, &mut errors);

    // 4. @contract 검증
    validate_contracts(file, &mut errors);

    // 5. @rule 검증
    validate_rules(file, &mut errors);

    // 6. 중복 이름 검증
    validate_unique_names(file, &mut errors);

    errors
}

/// @meta 블록 필수 필드 검증
fn validate_meta_required(file: &N2File, errors: &mut Vec<ValidationError>) {
    let meta_blocks: Vec<_> = file.blocks.iter().filter_map(|b| {
        if let Block::Meta(m) = b { Some(m) } else { None }
    }).collect();

    if meta_blocks.is_empty() {
        errors.push(ValidationError {
            block: "@meta".to_string(),
            field: "(missing)".to_string(),
            message: "@meta 블록은 필수입니다".to_string(),
            severity: Severity::Error,
        });
        return;
    }

    if meta_blocks.len() > 1 {
        errors.push(ValidationError {
            block: "@meta".to_string(),
            field: "(duplicate)".to_string(),
            message: "@meta 블록은 하나만 허용됩니다".to_string(),
            severity: Severity::Error,
        });
    }

    let meta = &meta_blocks[0];
    let required_fields = ["name", "enforce"];
    for req in &required_fields {
        if !meta.fields.iter().any(|f| f.key == *req) {
            errors.push(ValidationError {
                block: "@meta".to_string(),
                field: req.to_string(),
                message: format!("필수 필드 '{}'가 누락되었습니다", req),
                severity: Severity::Error,
            });
        }
    }

    // enforce 값 검증
    if let Some(enforce_field) = meta.fields.iter().find(|f| f.key == "enforce") {
        let valid_values = ["strict", "warn", "passive"];
        let val = match &enforce_field.value {
            Value::String(s) => s.clone(),
            Value::Identifier(s) => s.clone(),
            _ => String::new(),
        };
        if !valid_values.contains(&val.as_str()) {
            errors.push(ValidationError {
                block: "@meta".to_string(),
                field: "enforce".to_string(),
                message: format!("'{}' 는 유효하지 않습니다. strict|warn|passive 중 하나여야 합니다", val),
                severity: Severity::Error,
            });
        }
    }
}

/// @schema 정의 수집 (추후 타입 체크용)
fn collect_schema_defs(file: &N2File) -> HashMap<String, &SchemaDef> {
    let mut defs = HashMap::new();
    for block in &file.blocks {
        if let Block::Schema(schema) = block {
            for def in &schema.definitions {
                defs.insert(def.name.clone(), def);
            }
        }
    }
    defs
}

/// @workflow 검증
fn validate_workflows(file: &N2File, errors: &mut Vec<ValidationError>) {
    for block in &file.blocks {
        if let Block::Workflow(wf) = block {
            // 이름 필수
            if wf.name.is_empty() {
                errors.push(ValidationError {
                    block: "@workflow".to_string(),
                    field: "name".to_string(),
                    message: "워크플로우 이름이 필요합니다".to_string(),
                    severity: Severity::Error,
                });
            }

            // step이 최소 1개
            if wf.steps.is_empty() {
                errors.push(ValidationError {
                    block: format!("@workflow {}", wf.name),
                    field: "steps".to_string(),
                    message: "워크플로우에 최소 1개의 step이 필요합니다".to_string(),
                    severity: Severity::Error,
                });
            }

            // step 이름 중복 검사
            let mut step_names: Vec<&str> = Vec::new();
            for step in &wf.steps {
                if step_names.contains(&step.name.as_str()) {
                    errors.push(ValidationError {
                        block: format!("@workflow {}", wf.name),
                        field: format!("step {}", step.name),
                        message: format!("중복된 step 이름: '{}'", step.name),
                        severity: Severity::Error,
                    });
                }
                step_names.push(&step.name);
            }

            // depends_on 참조 검증
            for step in &wf.steps {
                for field in &step.fields {
                    if field.key == "depends_on" {
                        let dep = match &field.value {
                            Value::String(s) => s.clone(),
                            Value::Identifier(s) => s.clone(),
                            _ => String::new(),
                        };
                        if !dep.is_empty() && !step_names.contains(&dep.as_str()) {
                            errors.push(ValidationError {
                                block: format!("@workflow {}", wf.name),
                                field: format!("step {}", step.name),
                                message: format!("depends_on '{}' 는 존재하지 않는 step입니다", dep),
                                severity: Severity::Error,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// @contract 검증
fn validate_contracts(file: &N2File, errors: &mut Vec<ValidationError>) {
    for block in &file.blocks {
        if let Block::Contract(ct) = block {
            if ct.name.is_empty() {
                errors.push(ValidationError {
                    block: "@contract".to_string(),
                    field: "name".to_string(),
                    message: "계약 이름이 필요합니다".to_string(),
                    severity: Severity::Error,
                });
            }

            // transitions 검증: from/to 상태가 states에 정의되어 있는지
            if !ct.transitions.is_empty() && ct.states.is_none() {
                errors.push(ValidationError {
                    block: format!("@contract {}", ct.name),
                    field: "transitions".to_string(),
                    message: "transitions를 정의하려면 states도 정의해야 합니다".to_string(),
                    severity: Severity::Error,
                });
            }
        }
    }
}

/// @rule 검증
fn validate_rules(file: &N2File, errors: &mut Vec<ValidationError>) {
    for block in &file.blocks {
        if let Block::Rule(rule) = block {
            if rule.name.is_empty() {
                errors.push(ValidationError {
                    block: "@rule".to_string(),
                    field: "name".to_string(),
                    message: "규칙 이름이 필요합니다".to_string(),
                    severity: Severity::Error,
                });
            }

            // check도 없고 blacklist도 없으면 경고
            if rule.checks.is_empty() && rule.blacklist.is_empty() {
                errors.push(ValidationError {
                    block: format!("@rule {}", rule.name),
                    field: "body".to_string(),
                    message: "check 또는 blacklist가 비어있습니다. 규칙이 아무것도 검증하지 않습니다".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }
}

/// 블록 이름 중복 검증
fn validate_unique_names(file: &N2File, errors: &mut Vec<ValidationError>) {
    let mut names: HashMap<String, String> = HashMap::new();

    for block in &file.blocks {
        let (block_type, name) = match block {
            Block::Workflow(w) => ("@workflow", w.name.clone()),
            Block::Contract(c) => ("@contract", c.name.clone()),
            Block::Rule(r) => ("@rule", r.name.clone()),
            Block::Query(q) => ("@query", q.name.clone()),
            Block::Semantic(s) => ("@semantic", s.name.clone()),
            _ => continue,
        };

        if name.is_empty() { continue; }

        if let Some(existing_type) = names.get(&name) {
            errors.push(ValidationError {
                block: block_type.to_string(),
                field: "name".to_string(),
                message: format!("이름 '{}'가 {}에서 이미 사용되고 있습니다", name, existing_type),
                severity: Severity::Error,
            });
        } else {
            names.insert(name, block_type.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_n2;

    #[test]
    fn test_missing_meta() {
        let source = r#"
@workflow Test {
  step do_thing {
    action: test()
  }
}
"#;
        let file = parse_n2(source).unwrap();
        let errors = validate(&file);
        assert!(errors.iter().any(|e| e.block == "@meta" && e.severity == Severity::Error));
    }

    #[test]
    fn test_valid_file() {
        let source = r#"
@meta {
  name: "test"
  enforce: strict
}

@workflow Boot {
  step boot {
    action: n2_boot()
  }
}
"#;
        let file = parse_n2(source).unwrap();
        let errors = validate(&file);
        let error_count = errors.iter().filter(|e| e.severity == Severity::Error).count();
        assert_eq!(error_count, 0, "에러가 없어야 합니다: {:?}", errors);
    }

    #[test]
    fn test_duplicate_step_names() {
        let source = r#"
@meta {
  name: "test"
  enforce: strict
}

@workflow Boot {
  step boot {
    action: first()
  }
  step boot {
    action: second()
  }
}
"#;
        let file = parse_n2(source).unwrap();
        let errors = validate(&file);
        assert!(errors.iter().any(|e| e.message.contains("중복된 step 이름")));
    }
}
