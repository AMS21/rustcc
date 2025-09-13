pub mod binary_operator;
pub mod expression;
pub mod function_definition;
pub mod statement;
pub mod translation_unit;
pub mod unary_operator;

use rustcc_source::source_range::SourceRange;

fn ast_source_range_to_string(range: &SourceRange<'_>) -> String {
    if range.begin == range.end {
        return format!("{}:{}", range.begin.line, range.begin.column);
    }

    format!(
        "{}:{}-{}:{}",
        range.begin.line, range.begin.column, range.end.line, range.end.column
    )
}
