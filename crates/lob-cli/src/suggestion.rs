//! Error suggestion system for better error messages

/// A suggested fix for a compilation error
pub struct ErrorSuggestion {
    /// Description of the problem
    pub problem: String,
    /// List of potential fixes
    pub fixes: Vec<String>,
}

/// Detect common error patterns and provide helpful suggestions
pub fn get_suggestion(stderr: &str, user_expr: Option<&str>) -> Option<ErrorSuggestion> {
    // String comparison errors (more general patterns)
    if (stderr.contains("mismatched types") || stderr.contains("PartialOrd"))
        && ((stderr.contains("String") && stderr.contains("integer"))
            || (stderr.contains("&String") && stderr.contains("integer"))
            || (stderr.contains("expected `&String`") && stderr.contains("found integer")))
    {
        return Some(ErrorSuggestion {
            problem: "Cannot compare string with number".to_string(),
            fixes: vec![
                "Parse to number first: x.parse::<i32>().unwrap()".to_string(),
                "Compare string lengths instead: x.len() > 5".to_string(),
                "Compare as strings: x > \"5\"".to_string(),
            ],
        });
    }

    // Cannot find function
    if stderr.contains("cannot find function") {
        if let Some(expr) = user_expr {
            if expr.contains(".parse_csv") {
                return Some(ErrorSuggestion {
                    problem: "parse_csv() is not a method".to_string(),
                    fixes: vec!["Use --parse-csv flag: lob --parse-csv '_.filter(...)'".to_string()],
                });
            }
        }
        return Some(ErrorSuggestion {
            problem: "Unknown function or method".to_string(),
            fixes: vec![
                "Check available operations: filter, map, take, skip, count, sum".to_string(),
                "See docs: https://github.com/olirice/lob".to_string(),
            ],
        });
    }

    // Type mismatch in closures
    if stderr.contains("mismatched types") && stderr.contains("closure") {
        return Some(ErrorSuggestion {
            problem: "Type mismatch in closure".to_string(),
            fixes: vec![
                "Check your closure parameter types".to_string(),
                "Use explicit types: |x: &Type| if inference fails".to_string(),
            ],
        });
    }

    // Cannot index with &str
    if stderr.contains("cannot index") && stderr.contains("with `&str`") {
        return Some(ErrorSuggestion {
            problem: "Cannot index string with string".to_string(),
            fixes: vec![
                "For CSV: use --parse-csv flag to parse files".to_string(),
                "Access columns with: row[\"column_name\"]".to_string(),
            ],
        });
    }

    // Missing unwrap
    if stderr.contains("Option<") && stderr.contains("expected") {
        return Some(ErrorSuggestion {
            problem: "Operation returns Option - need to unwrap".to_string(),
            fixes: vec![
                "Extract value: value.unwrap()".to_string(),
                "With fallback: value.unwrap_or(default)".to_string(),
            ],
        });
    }

    // Iterator not implemented
    if stderr.contains("not an iterator")
        || (stderr.contains("doesn't implement") && stderr.contains("Iterator"))
    {
        return Some(ErrorSuggestion {
            problem: "Value is not an iterator".to_string(),
            fixes: vec![
                "Create iterator: value.iter()".to_string(),
                "Check if result is terminal (count, sum return values, not iterators)".to_string(),
            ],
        });
    }

    None
}
