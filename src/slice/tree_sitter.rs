use tree_sitter::{Language, Parser};

pub struct TsSlice {
    pub language: Language,
}

pub fn rust_lang() -> TsSlice {
    TsSlice {
        language: tree_sitter_rust::LANGUAGE.into(),
    }
}

pub fn python_lang() -> TsSlice {
    TsSlice {
        language: tree_sitter_python::LANGUAGE.into(),
    }
}

pub fn json_lang() -> TsSlice {
    TsSlice {
        language: tree_sitter_json::LANGUAGE.into(),
    }
}

pub fn detect_language(path: &str) -> Option<TsSlice> {
    if path.ends_with(".rs") {
        Some(rust_lang())
    } else if path.ends_with(".py") {
        Some(python_lang())
    } else if path.ends_with(".json") {
        Some(json_lang())
    } else {
        None
    }
}

pub fn extract_changed_vars(content: &str, path: &str) -> Vec<String> {
    let lang = match detect_language(path) {
        Some(l) => l,
        None => return text_extract_vars(content),
    };
    let mut parser = Parser::new();
    if parser.set_language(&lang.language).is_err() {
        return text_extract_vars(content);
    }
    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return text_extract_vars(content),
    };
    let mut vars = Vec::new();
    extract_vars_recursive(tree.root_node(), content, &mut vars);
    vars.sort();
    vars.dedup();
    vars
}

fn extract_vars_recursive(node: tree_sitter::Node, content: &str, vars: &mut Vec<String>) {
    let kind = node.kind();
    if (kind == "let_declaration" || kind == "assignment")
        && let Some(name_node) = node.child_by_field_name("name")
        && let Ok(text) = name_node.utf8_text(content.as_bytes())
    {
        vars.push(text.to_string());
    }
    if (kind == "identifier" || kind == "field_identifier")
        && let Ok(text) = node.utf8_text(content.as_bytes())
    {
        vars.push(text.to_string());
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        extract_vars_recursive(child, content, vars);
    }
}

pub fn extract_function_calls(content: &str, path: &str) -> Vec<String> {
    let lang = match detect_language(path) {
        Some(l) => l,
        None => return text_extract_calls(content),
    };
    let mut parser = Parser::new();
    if parser.set_language(&lang.language).is_err() {
        return text_extract_calls(content);
    }
    let tree = match parser.parse(content, None) {
        Some(t) => t,
        None => return text_extract_calls(content),
    };
    let mut calls = Vec::new();
    extract_calls_recursive(tree.root_node(), content, &mut calls);
    calls.sort();
    calls.dedup();
    calls
}

fn extract_calls_recursive(node: tree_sitter::Node, content: &str, calls: &mut Vec<String>) {
    let kind = node.kind();
    if (kind == "call_expression" || kind == "macro_invocation")
        && let Some(func) = node.child_by_field_name("function")
        && let Ok(text) = func.utf8_text(content.as_bytes())
    {
        calls.push(text.to_string());
    }
    let mut cursor = node.walk();
    for child in node.named_children(&mut cursor) {
        extract_calls_recursive(child, content, calls);
    }
}

fn text_extract_vars(content: &str) -> Vec<String> {
    super::ast::extract_changed_vars(content)
}

fn text_extract_calls(content: &str) -> Vec<String> {
    super::ast::extract_function_calls(content)
}
