use crate::function_definition::FunctionDefinition;

// TODO: Should the translation unit have a file name field?

#[derive(Debug, Clone, Hash, Default)]
pub struct TranslationUnit<'a> {
    pub function: Vec<FunctionDefinition<'a>>,
}

impl TranslationUnit<'_> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            function: Vec::new(),
        }
    }

    #[must_use]
    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str("TranslationUnit\n");

        // Dump all function definitions
        for function in &self.function {
            result.push_str(&function.dump(1));
        }

        result
    }
}
