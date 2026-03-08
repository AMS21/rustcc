use crate::{declaration::Declaration, statement::Statement};

#[derive(Debug, Clone, Hash)]
pub enum BlockItem<'a> {
    Statement(Statement<'a>),
    Declaration(Declaration<'a>),
}

#[derive(Debug, Clone, Hash)]
pub struct FunctionDefinition<'a> {
    pub name: String,
    pub body: Vec<BlockItem<'a>>,
    // TODO: Source Ranges for the function definition
}

impl<'a> FunctionDefinition<'a> {
    #[must_use]
    pub fn new<S: Into<String>>(name: S, body: Vec<BlockItem<'a>>) -> Self {
        Self {
            name: name.into(),
            body,
        }
    }

    #[must_use]
    pub fn dump(&self, depth: usize) -> String {
        let str = format!(
            "{}FunctionDefinition \"{}\"\n",
            "  ".repeat(depth),
            self.name,
        );

        let str = self.body.iter().fold(str, |acc, item| {
            let item_str = match item {
                BlockItem::Statement(stmt) => stmt.dump(depth + 1),
                BlockItem::Declaration(decl) => decl.dump(depth + 1),
            };

            format!("{acc}{item_str}\n")
        });

        str.trim_end().to_string()
    }
}
