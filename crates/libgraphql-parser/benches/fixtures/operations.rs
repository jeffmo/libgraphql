use std::fmt::Write;

/// Generates a query with deeply nested selection sets.
///
/// At each level, the query selects an `id` field and a `child`
/// field that contains the next nesting level. The innermost level
/// selects `id` and `name`.
pub fn deeply_nested_query(depth: usize) -> String {
    let mut out = String::with_capacity(depth * 30);
    out.push_str("query DeeplyNested {\n");
    for level in 0..depth {
        let indent = "  ".repeat(level + 1);
        if level == 0 {
            writeln!(out, "{indent}root {{").unwrap();
        } else {
            writeln!(out, "{indent}child {{").unwrap();
        }
        writeln!(out, "{indent}  id").unwrap();
    }
    // Innermost fields
    let inner_indent = "  ".repeat(depth + 1);
    writeln!(out, "{inner_indent}name").unwrap();
    // Close all braces
    for level in (0..depth).rev() {
        let indent = "  ".repeat(level + 1);
        writeln!(out, "{indent}}}").unwrap();
    }
    out.push_str("}\n");
    out
}

/// Generates a document containing `count` named query operations.
///
/// Each query has a unique name and selects a small set of fields
/// including its index, providing a realistic multi-operation
/// document.
pub fn many_operations(count: usize) -> String {
    let mut out = String::with_capacity(count * 80);
    for i in 0..count {
        writeln!(out, "query Operation{i}($id: ID!) {{").unwrap();
        writeln!(out, "  node(id: $id) {{").unwrap();
        writeln!(out, "    id").unwrap();
        writeln!(out, "    name").unwrap();
        writeln!(out, "    field{i}: description").unwrap();
        writeln!(out, "  }}").unwrap();
        writeln!(out, "}}\n").unwrap();
    }
    out
}
