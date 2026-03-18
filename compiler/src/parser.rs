// .n2 파서 — pest PEG 파싱 결과를 N2AST로 변환
use pest::Parser;
use pest_derive::Parser;

use crate::ast::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct N2Parser;

/// .n2 소스 코드를 파싱하여 N2File AST를 반환
pub fn parse_n2(source: &str) -> Result<N2File, String> {
    let pairs = N2Parser::parse(Rule::n2_file, source)
        .map_err(|e| format!("파싱 에러: {}", e))?;

    let mut blocks = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::n2_file => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::block => {
                            let block_inner = inner_pair.into_inner().next().unwrap();
                            let block = parse_block(block_inner)?;
                            blocks.push(block);
                        }
                        Rule::EOI => {}
                        _ => {}
                    }
                }
            }
            Rule::block => {
                let inner = pair.into_inner().next().unwrap();
                let block = parse_block(inner)?;
                blocks.push(block);
            }
            Rule::EOI => {}
            _ => {}
        }
    }

    Ok(N2File { blocks })
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> Result<Block, String> {
    match pair.as_rule() {
        Rule::meta_block => parse_meta_block(pair).map(Block::Meta),
        Rule::import_block => parse_import_block(pair).map(Block::Import),
        Rule::schema_block => parse_schema_block(pair).map(Block::Schema),
        Rule::contract_block => parse_contract_block(pair).map(Block::Contract),
        Rule::rule_block => parse_rule_block(pair).map(Block::Rule),
        Rule::workflow_block => parse_workflow_block(pair).map(Block::Workflow),
        Rule::query_block => parse_query_block(pair).map(Block::Query),
        Rule::semantic_block => parse_semantic_block(pair).map(Block::Semantic),
        _ => Err(format!("알 수 없는 블록 타입: {:?}", pair.as_rule())),
    }
}

// ── @meta 파싱 ──
fn parse_meta_block(pair: pest::iterators::Pair<Rule>) -> Result<MetaBlock, String> {
    let mut fields = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::meta_field {
            fields.push(parse_field(inner)?);
        }
    }
    Ok(MetaBlock { fields })
}

// ── @import 파싱 ──
fn parse_import_block(pair: pest::iterators::Pair<Rule>) -> Result<ImportBlock, String> {
    let mut imports = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::import_stmt {
            let mut from = String::new();
            let mut names = Vec::new();
            for part in inner.into_inner() {
                match part.as_rule() {
                    Rule::string_literal => from = strip_quotes(part.as_str()),
                    Rule::import_list => {
                        for id in part.into_inner() {
                            if id.as_rule() == Rule::identifier {
                                names.push(id.as_str().to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            imports.push(ImportStmt { from, names });
        }
    }
    Ok(ImportBlock { imports })
}

// ── @schema 파싱 ──
fn parse_schema_block(pair: pest::iterators::Pair<Rule>) -> Result<SchemaBlock, String> {
    let mut definitions = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::schema_def {
            let mut name = String::new();
            let mut is_enum = false;
            let mut fields = Vec::new();
            for part in inner.into_inner() {
                match part.as_rule() {
                    Rule::identifier => {
                        if name.is_empty() {
                            name = part.as_str().to_string();
                        } else if part.as_str() == "enum" {
                            is_enum = true;
                        }
                    }
                    Rule::schema_field => {
                        let mut fname = String::new();
                        let mut type_expr = None;
                        let mut attributes = Vec::new();
                        for fp in part.into_inner() {
                            match fp.as_rule() {
                                Rule::identifier => fname = fp.as_str().to_string(),
                                Rule::type_expr => type_expr = Some(fp.as_str().to_string()),
                                Rule::field_attrs => {
                                    for attr in fp.into_inner() {
                                        attributes.push(attr.as_str().to_string());
                                    }
                                }
                                _ => {}
                            }
                        }
                        fields.push(SchemaField { name: fname, type_expr, attributes });
                    }
                    _ => {}
                }
            }
            definitions.push(SchemaDef { name, is_enum, fields });
        }
    }
    Ok(SchemaBlock { definitions })
}

// ── @contract 파싱 ──
fn parse_contract_block(pair: pest::iterators::Pair<Rule>) -> Result<ContractBlock, String> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut preconditions = Vec::new();
    let mut postconditions = Vec::new();
    let mut invariants = Vec::new();
    let mut recovery = Vec::new();
    let mut states = None;
    let mut transitions = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::contract_body => {
                let body = inner.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::contract_field => fields.push(parse_field(body)?),
                    Rule::precondition_block => {
                        for stmt in body.into_inner() {
                            if stmt.as_rule() == Rule::condition_stmt {
                                preconditions.push(parse_condition_stmt(stmt));
                            }
                        }
                    }
                    Rule::postcondition_block => {
                        for stmt in body.into_inner() {
                            if stmt.as_rule() == Rule::condition_stmt {
                                postconditions.push(parse_condition_stmt(stmt));
                            }
                        }
                    }
                    Rule::invariant_block => {
                        for stmt in body.into_inner() {
                            invariants.push(stmt.as_str().to_string());
                        }
                    }
                    Rule::recovery_block => {
                        for stmt in body.into_inner() {
                            recovery.push(stmt.as_str().to_string());
                        }
                    }
                    Rule::states_block => {
                        for part in body.into_inner() {
                            if part.as_rule() == Rule::identifier {
                                states = Some(part.as_str().to_string());
                            }
                        }
                    }
                    Rule::transitions_block => {
                        for stmt in body.into_inner() {
                            if stmt.as_rule() == Rule::transition_stmt {
                                let parts: Vec<_> = stmt.into_inner().collect();
                                if parts.len() >= 3 {
                                    transitions.push(TransitionStmt {
                                        from: parts[0].as_str().to_string(),
                                        to: parts[1].as_str().to_string(),
                                        on_event: parts[2].as_str().to_string(),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(ContractBlock { name, fields, preconditions, postconditions, invariants, recovery, states, transitions })
}

// ── @rule 파싱 ──
fn parse_rule_block(pair: pest::iterators::Pair<Rule>) -> Result<RuleBlock, String> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut checks = Vec::new();
    let mut blacklist = Vec::new();
    let mut on_match = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::rule_body => {
                let body = inner.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::rule_field => fields.push(parse_field(body)?),
                    Rule::check_block => {
                        for stmt in body.into_inner() {
                            if stmt.as_rule() == Rule::condition_stmt {
                                checks.push(parse_condition_stmt(stmt));
                            }
                        }
                    }
                    Rule::blacklist_block => {
                        for regex in body.into_inner() {
                            if regex.as_rule() == Rule::regex_list {
                                for r in regex.into_inner() {
                                    blacklist.push(r.as_str().to_string());
                                }
                            }
                        }
                    }
                    Rule::on_match_block => {
                        on_match.push(body.as_str().to_string());
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(RuleBlock { name, fields, checks, blacklist, on_match })
}

// ── @workflow 파싱 ──
fn parse_workflow_block(pair: pest::iterators::Pair<Rule>) -> Result<WorkflowBlock, String> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut contracts = Vec::new();
    let mut steps = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::workflow_body => {
                let body = inner.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::workflow_field => fields.push(parse_field(body)?),
                    Rule::apply_contracts => {
                        for part in body.into_inner() {
                            if part.as_rule() == Rule::identifier_list {
                                for id in part.into_inner() {
                                    contracts.push(id.as_str().to_string());
                                }
                            }
                        }
                    }
                    Rule::step_block => {
                        steps.push(parse_step_block(body)?);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(WorkflowBlock { name, fields, contracts, steps })
}

fn parse_step_block(pair: pest::iterators::Pair<Rule>) -> Result<StepBlock, String> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut output = None;
    let mut expects = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::step_body => {
                let body = inner.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::step_field => fields.push(parse_field(body)?),
                    Rule::output_binding => {
                        for part in body.into_inner() {
                            if part.as_rule() == Rule::variable {
                                output = Some(part.as_str().to_string());
                            }
                        }
                    }
                    Rule::expect_block => {
                        for stmt in body.into_inner() {
                            if stmt.as_rule() == Rule::expect_stmt {
                                let text = stmt.as_str().to_string();
                                let parts: Vec<&str> = text.splitn(2, "=>").collect();
                                expects.push(ExpectStmt {
                                    condition: parts.first().unwrap_or(&"").trim().to_string(),
                                    action: parts.get(1).unwrap_or(&"").trim().to_string(),
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(StepBlock { name, fields, output, expects })
}

// ── @query 파싱 ──
fn parse_query_block(pair: pest::iterators::Pair<Rule>) -> Result<QueryBlock, String> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut sql = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::query_body => {
                let body = inner.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::query_field => fields.push(parse_field(body)?),
                    Rule::sql_block => {
                        for part in body.into_inner() {
                            if part.as_rule() == Rule::sql_content {
                                sql = Some(part.as_str().trim().to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(QueryBlock { name, fields, sql })
}

// ── @semantic 파싱 ──
fn parse_semantic_block(pair: pest::iterators::Pair<Rule>) -> Result<SemanticBlock, String> {
    let mut name = String::new();
    let mut fields = Vec::new();
    let mut indexes = Vec::new();
    let mut aliases = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => name = inner.as_str().to_string(),
            Rule::semantic_body => {
                let body = inner.into_inner().next().unwrap();
                match body.as_rule() {
                    Rule::semantic_field => fields.push(parse_field(body)?),
                    Rule::index_block => {
                        let mut idx_name = String::new();
                        let mut idx_fields = Vec::new();
                        for part in body.into_inner() {
                            match part.as_rule() {
                                Rule::identifier => idx_name = part.as_str().to_string(),
                                Rule::step_field => idx_fields.push(parse_field(part)?),
                                _ => {}
                            }
                        }
                        indexes.push(IndexDef { name: idx_name, fields: idx_fields });
                    }
                    Rule::aliases_block => {
                        for stmt in body.into_inner() {
                            if stmt.as_rule() == Rule::alias_stmt {
                                let parts: Vec<_> = stmt.into_inner().collect();
                                if parts.len() >= 2 {
                                    aliases.push(AliasDef {
                                        pattern: strip_quotes(parts[0].as_str()),
                                        target: parts[1].as_str().to_string(),
                                    });
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(SemanticBlock { name, fields, indexes, aliases })
}

// ── 공통 헬퍼 ──
fn parse_field(pair: pest::iterators::Pair<Rule>) -> Result<Field, String> {
    let mut key = String::new();
    let mut val = Value::String(String::new());

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::identifier => key = inner.as_str().to_string(),
            Rule::value => val = parse_value(inner),
            _ => {}
        }
    }

    Ok(Field { key, value: val })
}

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Value {
    let inner = pair.into_inner().next();
    match inner {
        Some(p) => match p.as_rule() {
            Rule::string_literal | Rule::template_string => Value::String(strip_quotes(p.as_str())),
            Rule::number => Value::Number(p.as_str().parse().unwrap_or(0.0)),
            Rule::boolean => Value::Bool(p.as_str() == "true"),
            Rule::identifier => Value::Identifier(p.as_str().to_string()),
            Rule::variable => Value::String(p.as_str().to_string()),
            Rule::duration => Value::String(p.as_str().to_string()),
            Rule::regex_literal => Value::String(p.as_str().to_string()),
            Rule::array_literal => {
                let items: Vec<Value> = p.into_inner()
                    .filter(|v| v.as_rule() == Rule::value)
                    .map(parse_value)
                    .collect();
                Value::Array(items)
            }
            Rule::function_call => Value::String(p.as_str().to_string()),
            _ => Value::String(p.as_str().to_string()),
        },
        None => Value::String(String::new()),
    }
}

fn parse_condition_stmt(pair: pest::iterators::Pair<Rule>) -> ConditionStmt {
    let text = pair.as_str().to_string();
    let parts: Vec<&str> = text.splitn(2, "=>").collect();
    ConditionStmt {
        expression: parts.first().unwrap_or(&"").trim().to_string(),
        message: strip_quotes(parts.get(1).unwrap_or(&"").trim()),
    }
}

fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if s.starts_with("\"\"\"") && s.ends_with("\"\"\"") {
        s[3..s.len()-3].to_string()
    } else if s.starts_with('"') && s.ends_with('"') {
        s[1..s.len()-1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_meta() {
        let source = r#"
@meta {
  name: "test"
  version: "1.0.0"
  enforce: strict
}
"#;
        let result = parse_n2(source);
        assert!(result.is_ok(), "파싱 실패: {:?}", result.err());
        let file = result.unwrap();
        assert_eq!(file.blocks.len(), 1);
    }

    #[test]
    fn test_parse_workflow() {
        let source = r#"
@meta {
  name: "test-workflow"
  target: workflow
  enforce: strict
}

@workflow Boot {
  trigger: session_start
  enforce: strict

  step boot {
    action: n2_boot()
    required: true
    timeout: 15s
    output -> $BOOT
  }

  step greet {
    depends_on: boot
    action: compose_response()
  }
}
"#;
        let result = parse_n2(source);
        assert!(result.is_ok(), "파싱 실패: {:?}", result.err());
        let file = result.unwrap();
        assert_eq!(file.blocks.len(), 2);
    }
}
