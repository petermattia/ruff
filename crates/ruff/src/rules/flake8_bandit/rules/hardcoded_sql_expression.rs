use once_cell::sync::Lazy;
use regex::Regex;
use rustpython_parser::ast::{self, Expr, Operator, Ranged};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::helpers::any_over_expr;

use crate::checkers::ast::Checker;

use super::super::helpers::string_literal;

static SQL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(select\s.+\sfrom\s|delete\s+from\s|(insert|replace)\s.+\svalues\s|update\s.+\sset\s)")
        .unwrap()
});

/// ## What it does
/// Checks for strings that resemble SQL statements involved in some form
/// string building operation.
///
/// ## Why is this bad?
/// SQL injection is a common attack vector for web applications. Directly
/// interpolating user input into SQL statements should always be avoided.
/// Instead, favor parameterized queries, in which the SQL statement is
/// provided separately from its parameters, as supported by `psycopg3`
/// and other database drivers and ORMs.
///
/// ## Example
/// ```python
/// query = "DELETE FROM foo WHERE id = '%s'" % identifier
/// ```
///
/// ## References
/// - [B608: Test for SQL injection](https://bandit.readthedocs.io/en/latest/plugins/b608_hardcoded_sql_expressions.html)
/// - [psycopg3: Server-side binding](https://www.psycopg.org/psycopg3/docs/basic/from_pg2.html#server-side-binding)
#[violation]
pub struct HardcodedSQLExpression;

impl Violation for HardcodedSQLExpression {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Possible SQL injection vector through string-based query construction")
    }
}

fn has_string_literal(expr: &Expr) -> bool {
    string_literal(expr).is_some()
}

fn matches_sql_statement(string: &str) -> bool {
    SQL_REGEX.is_match(string)
}

fn unparse_string_format_expression(checker: &mut Checker, expr: &Expr) -> Option<String> {
    match expr {
        // "select * from table where val = " + "str" + ...
        // "select * from table where val = %s" % ...
        Expr::BinOp(ast::ExprBinOp {
            op: Operator::Add | Operator::Mod,
            ..
        }) => {
            let Some(parent) = checker.semantic().expr_parent() else {
                if any_over_expr(expr, &has_string_literal) {
                    return Some(checker.generator().expr(expr));
                }
                return None;
            };
            // Only evaluate the full BinOp, not the nested components.
            let Expr::BinOp(_) = parent else {
                if any_over_expr(expr, &has_string_literal) {
                    return Some(checker.generator().expr(expr));
                }
                return None;
            };
            None
        }
        Expr::Call(ast::ExprCall { func, .. }) => {
            let Expr::Attribute(ast::ExprAttribute { attr, value, .. }) = func.as_ref() else {
                return None;
            };
            // "select * from table where val = {}".format(...)
            if attr == "format" && string_literal(value).is_some() {
                return Some(checker.generator().expr(expr));
            };
            None
        }
        // f"select * from table where val = {val}"
        Expr::JoinedStr(_) => Some(checker.generator().expr(expr)),
        _ => None,
    }
}

/// S608
pub(crate) fn hardcoded_sql_expression(checker: &mut Checker, expr: &Expr) {
    match unparse_string_format_expression(checker, expr) {
        Some(string) if matches_sql_statement(&string) => {
            checker
                .diagnostics
                .push(Diagnostic::new(HardcodedSQLExpression, expr.range()));
        }
        _ => (),
    }
}
