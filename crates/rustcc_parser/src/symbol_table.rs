use rustc_hash::FxHashMap;
use rustcc_source::source_range::SourceRange;

/// Metadata about a declared symbol (variable, typedef, etc.).
#[derive(Debug, Clone, Copy)]
pub struct SymbolInfo<'a> {
    /// The source range of the declaration.
    pub declaration_range: SourceRange<'a>,
}

/// A scoped symbol table backed by a stack of hash maps.
///
/// Each scope is a separate map. Lookups search from the innermost scope
/// outward, while insertions always target the current (innermost) scope.
#[derive(Debug, Clone)]
pub struct SymbolTable<'a> {
    scopes: Vec<FxHashMap<String, SymbolInfo<'a>>>,
}

impl<'a> SymbolTable<'a> {
    #[must_use]
    pub const fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    /// Push a new (empty) scope onto the stack.
    pub fn push_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    /// Pop the innermost scope, discarding all its entries.
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    /// Try to insert a symbol into the current scope.
    ///
    /// Returns `Some(previous_info)` if the name was **already declared** in
    /// the current scope (the original entry is kept). Returns `None` on
    /// successful insertion.
    #[expect(clippy::expect_used, clippy::unwrap_in_result)]
    pub fn insert(&mut self, name: String, info: SymbolInfo<'a>) -> Option<SymbolInfo<'a>> {
        let scope = self.scopes.last_mut().expect("no active scope");
        if let Some(existing) = scope.get(&name) {
            return Some(*existing);
        }
        scope.insert(name, info);
        None
    }

    /// Look up a symbol, searching from the innermost scope outward.
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&SymbolInfo<'a>> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info);
            }
        }
        None
    }
}

impl Default for SymbolTable<'_> {
    fn default() -> Self {
        Self::new()
    }
}
