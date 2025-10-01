use rustcc_source::source_range::SourceRange;

use crate::expression::Expression;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Declaration<'a> {
    pub name: String,
    pub initializer: Option<Expression<'a>>,
    pub range: SourceRange<'a>,
}

impl<'a> Declaration<'a> {
    #[must_use]
    pub const fn new(
        name: String,
        initializer: Option<Expression<'a>>,
        range: SourceRange<'a>,
    ) -> Self {
        Self {
            name,
            initializer,
            range,
        }
    }

    #[must_use]
    pub fn dump(&self, depth: usize) -> String {
        self.initializer.as_ref().map_or_else(
            || {
                format!(
                    "{}Declaration ({})\n{}<uninitialized>",
                    "  ".repeat(depth),
                    self.name,
                    "  ".repeat(depth + 1)
                )
            },
            |initializer| {
                format!(
                    "{}Declaration ({})\n{}",
                    "  ".repeat(depth),
                    self.name,
                    initializer.dump(depth + 1)
                )
            },
        )
    }
}
