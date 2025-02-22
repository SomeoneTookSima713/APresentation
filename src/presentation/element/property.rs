#[derive(Clone)]
pub enum Property<T: rhai::Variant + Clone> {
    Literal(T),
    Script(rhai::AST)
}

impl<T: rhai::Variant + Clone> Property<T> {
    pub fn evaluate(&self, scope: &mut rhai::Scope<'static>, engine: &rhai::Engine) -> Option<T> {
        match self {
            Self::Literal(v) => Some(v.clone()),
            Self::Script(ast) => engine.eval_ast_with_scope::<T>(scope, ast).ok()
        }
    }
}