use crate::statement::Statement;

#[derive(Debug, Clone, Hash)]
pub struct FunctionDefinition<'a> {
    pub name: String,
    pub body: Statement<'a>,
    // TODO: Source Ranges for the function definition
}

impl<'a> FunctionDefinition<'a> {
    #[must_use]
    pub fn new<S: Into<String>>(name: S, body: Statement<'a>) -> Self {
        Self {
            name: name.into(),
            body,
        }
    }

    #[must_use]
    pub fn dump(&self, depth: usize) -> String {
        format!(
            "{}FunctionDefinition \"{}\"\n{}",
            "  ".repeat(depth),
            self.name,
            self.body.dump(depth + 1)
        )
    }
}
