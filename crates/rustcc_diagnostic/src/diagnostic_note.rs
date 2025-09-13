use rustcc_source::source_range::SourceRange;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DiagnosticNote<'a> {
    pub source_range: SourceRange<'a>,
    pub message: String,
}
