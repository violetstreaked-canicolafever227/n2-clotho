// SQL 쿼리 엔진 — @query 블록의 SQL 실행 + 도구/규칙/상태 가상 테이블
use crate::ast::*;
use std::collections::HashMap;

/// 가상 레지스트리: .n2 파일에서 추출한 도구/규칙/계약 정보를 SQL로 쿼리 가능하게
#[derive(Debug)]
pub struct N2Registry {
    pub tools: Vec<ToolEntry>,
    pub rules: Vec<RuleEntry>,
    pub contracts: Vec<ContractEntry>,
    pub workflows: Vec<WorkflowEntry>,
}

#[derive(Debug, Clone)]
pub struct ToolEntry {
    pub name: String,
    pub category: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct RuleEntry {
    pub name: String,
    pub scope: String,
    pub enforce: String,
    pub check_count: usize,
    pub blacklist_count: usize,
}

#[derive(Debug, Clone)]
pub struct ContractEntry {
    pub name: String,
    pub scope: String,
    pub state_count: usize,
    pub transition_count: usize,
}

#[derive(Debug, Clone)]
pub struct WorkflowEntry {
    pub name: String,
    pub trigger: String,
    pub enforce: String,
    pub step_count: usize,
}

/// SQL 쿼리 결과
#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl std::fmt::Display for QueryResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rows.is_empty() {
            return write!(f, "(결과 없음)");
        }

        // 컬럼 폭 계산
        let mut widths: Vec<usize> = self.columns.iter().map(|c| c.len()).collect();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        // 헤더
        let header: String = self.columns.iter().enumerate()
            .map(|(i, c)| format!("{:width$}", c, width = widths[i]))
            .collect::<Vec<_>>().join(" | ");
        writeln!(f, "{}", header)?;

        // 구분선
        let separator: String = widths.iter()
            .map(|w| "-".repeat(*w))
            .collect::<Vec<_>>().join("-+-");
        writeln!(f, "{}", separator)?;

        // 행
        for row in &self.rows {
            let line: String = row.iter().enumerate()
                .map(|(i, c)| format!("{:width$}", c, width = widths.get(i).copied().unwrap_or(10)))
                .collect::<Vec<_>>().join(" | ");
            writeln!(f, "{}", line)?;
        }

        Ok(())
    }
}

impl N2Registry {
    /// N2File에서 레지스트리 구축
    pub fn from_file(file: &N2File) -> Self {
        let mut tools = Vec::new();
        let mut rules = Vec::new();
        let mut contracts = Vec::new();
        let mut workflows = Vec::new();

        for block in &file.blocks {
            match block {
                Block::Rule(r) => {
                    let scope = get_field_str(&r.fields, "scope");
                    let enforce = get_field_str(&r.fields, "enforce");
                    rules.push(RuleEntry {
                        name: r.name.clone(),
                        scope,
                        enforce,
                        check_count: r.checks.len(),
                        blacklist_count: r.blacklist.len(),
                    });
                }
                Block::Contract(c) => {
                    let scope = get_field_str(&c.fields, "scope");
                    contracts.push(ContractEntry {
                        name: c.name.clone(),
                        scope,
                        state_count: c.transitions.iter()
                            .flat_map(|t| vec![t.from.clone(), t.to.clone()])
                            .collect::<std::collections::HashSet<_>>().len(),
                        transition_count: c.transitions.len(),
                    });
                }
                Block::Workflow(w) => {
                    let trigger = get_field_str(&w.fields, "trigger");
                    let enforce = get_field_str(&w.fields, "enforce");
                    workflows.push(WorkflowEntry {
                        name: w.name.clone(),
                        trigger,
                        enforce,
                        step_count: w.steps.len(),
                    });
                }
                _ => {}
            }
        }

        N2Registry { tools, rules, contracts, workflows }
    }

    /// 간단한 SQL-like 쿼리 실행 (SELECT ... FROM ... WHERE 패턴)
    pub fn execute_query(&self, sql: &str) -> Result<QueryResult, String> {
        let sql_lower = sql.to_lowercase().trim().to_string();

        // 테이블 감지
        if sql_lower.contains("from rules") || sql_lower.contains("from rule") {
            Ok(self.query_rules())
        } else if sql_lower.contains("from contracts") || sql_lower.contains("from contract") {
            Ok(self.query_contracts())
        } else if sql_lower.contains("from workflows") || sql_lower.contains("from workflow") {
            Ok(self.query_workflows())
        } else if sql_lower.contains("from tools") || sql_lower.contains("from tool") {
            Ok(self.query_tools())
        } else {
            Err(format!("지원하지 않는 테이블입니다. 사용 가능: rules, contracts, workflows, tools"))
        }
    }

    fn query_rules(&self) -> QueryResult {
        QueryResult {
            columns: vec!["name".into(), "scope".into(), "enforce".into(), "checks".into(), "blacklist".into()],
            rows: self.rules.iter().map(|r| vec![
                r.name.clone(), r.scope.clone(), r.enforce.clone(),
                r.check_count.to_string(), r.blacklist_count.to_string(),
            ]).collect(),
        }
    }

    fn query_contracts(&self) -> QueryResult {
        QueryResult {
            columns: vec!["name".into(), "scope".into(), "states".into(), "transitions".into()],
            rows: self.contracts.iter().map(|c| vec![
                c.name.clone(), c.scope.clone(),
                c.state_count.to_string(), c.transition_count.to_string(),
            ]).collect(),
        }
    }

    fn query_workflows(&self) -> QueryResult {
        QueryResult {
            columns: vec!["name".into(), "trigger".into(), "enforce".into(), "steps".into()],
            rows: self.workflows.iter().map(|w| vec![
                w.name.clone(), w.trigger.clone(), w.enforce.clone(),
                w.step_count.to_string(),
            ]).collect(),
        }
    }

    fn query_tools(&self) -> QueryResult {
        QueryResult {
            columns: vec!["name".into(), "category".into(), "description".into()],
            rows: self.tools.iter().map(|t| vec![
                t.name.clone(), t.category.clone(), t.description.clone(),
            ]).collect(),
        }
    }

    /// 레지스트리 요약
    pub fn summary(&self) -> String {
        format!("📦 Registry: {} rules, {} contracts, {} workflows, {} tools",
            self.rules.len(), self.contracts.len(), self.workflows.len(), self.tools.len())
    }
}

/// Field에서 문자열 값 추출 헬퍼
fn get_field_str(fields: &[Field], key: &str) -> String {
    fields.iter()
        .find(|f| f.key == key)
        .map(|f| match &f.value {
            Value::String(s) => s.clone(),
            Value::Identifier(s) => s.clone(),
            _ => String::new(),
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_n2;

    #[test]
    fn test_registry_from_file() {
        let source = r#"
@meta {
  name: "test"
  enforce: strict
}

@rule NoInstall {
  scope: command
  enforce: strict

  blacklist: [
    /npm install/,
    /pip install/
  ]
}

@contract SessionLifecycle {
  scope: session
  states: S

  transitions {
    IDLE -> WORKING : on start
    WORKING -> IDLE : on stop
  }
}

@workflow Boot {
  trigger: session_start
  enforce: strict

  step boot {
    action: n2_boot()
  }
}
"#;
        let file = parse_n2(source).unwrap();
        let registry = N2Registry::from_file(&file);

        assert_eq!(registry.rules.len(), 1);
        assert_eq!(registry.contracts.len(), 1);
        assert_eq!(registry.workflows.len(), 1);

        // SQL 쿼리 테스트
        let result = registry.execute_query("SELECT * FROM rules").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], "NoInstall");

        let result = registry.execute_query("SELECT * FROM contracts").unwrap();
        assert_eq!(result.rows.len(), 1);

        let result = registry.execute_query("SELECT * FROM workflows").unwrap();
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0][0], "Boot");
    }
}
