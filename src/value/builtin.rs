use std::fmt::{Debug, Formatter};
use crate::ast::PosExpression;
use crate::interpreter::eval_expression;
use crate::scope::ScopeRef;
use crate::value::{EvalContext, EvalResult, EvalValue};
use crate::value::error::{ErrorContext, EvalError};

/// One argument of a builtin call: either the unevaluated expression from the
/// call site (borrowed from the AST, nothing is allocated or cloned to pass it)
/// or an already evaluated value (calls through `map`, `apply`-style paths).
pub enum BuiltInFunctionArg<'a> {
    Expression(&'a PosExpression),
    Value(EvalValue),
}

/// The arguments of a builtin call. Builtins receive their arguments
/// unevaluated, so special forms (`if`, `let`, `fn`, `quote`) are ordinary
/// builtins; `evaluated()` evaluates on demand.
pub enum BuiltInFunctionArgs<'a> {
    Expressions(&'a [PosExpression]),
    Values(Vec<EvalValue>),
}

pub type InternalCallback = for<'a> fn(&ScopeRef, EvalContext, BuiltInFunctionArgs<'a>) -> EvalResult;

impl<'a> BuiltInFunctionArg<'a> {
    pub fn evaluated(&self, scope: &ScopeRef) -> EvalResult {
        match self {
            BuiltInFunctionArg::Expression(e) => eval_expression(EvalContext::none(), scope, e),
            BuiltInFunctionArg::Value(v) => Ok((v.clone(), EvalContext::none())),
        }
    }

    pub fn try_expression(&self, scope: &ScopeRef) -> Result<&'a PosExpression, ErrorContext> {
        match self {
            BuiltInFunctionArg::Expression(e) => Ok(e),
            BuiltInFunctionArg::Value(_) => Err(EvalError::InvalidType.trace(scope)),
        }
    }
}

impl<'a> BuiltInFunctionArgs<'a> {
    pub fn from(values: Vec<EvalValue>) -> BuiltInFunctionArgs<'a> {
        BuiltInFunctionArgs::Values(values)
    }

    pub fn len(&self) -> usize {
        match self {
            BuiltInFunctionArgs::Expressions(e) => e.len(),
            BuiltInFunctionArgs::Values(v) => v.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Iterate the arguments from position `from` on.
    pub fn iter_from(&self, from: usize) -> impl Iterator<Item = BuiltInFunctionArg<'a>> + '_ {
        (from..self.len()).map(move |i| self.get(i).unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = BuiltInFunctionArg<'a>> + '_ {
        self.iter_from(0)
    }

    pub fn get(&self, pos: usize) -> Option<BuiltInFunctionArg<'a>> {
        match self {
            BuiltInFunctionArgs::Expressions(e) => e.get(pos).map(BuiltInFunctionArg::Expression),
            BuiltInFunctionArgs::Values(v) => v.get(pos).cloned().map(BuiltInFunctionArg::Value),
        }
    }

    pub fn eval_all(self, scope: &ScopeRef) -> Result<Vec<EvalValue>, ErrorContext> {
        match self {
            BuiltInFunctionArgs::Values(v) => Ok(v),
            BuiltInFunctionArgs::Expressions(e) => e
                .iter()
                //discard ctx, cuz who cares
                .map(|exp| eval_expression(EvalContext::none(), scope, exp).map(|v| v.0))
                .collect(),
        }
    }

    pub fn try_pos(&self, scope: &ScopeRef, pos: usize) -> Result<BuiltInFunctionArg<'a>, ErrorContext> {
        match self.get(pos) {
            Some(v) => Ok(v),
            None => Err(EvalError::MissingArgument.trace(scope)),
        }
    }
}

pub struct BuiltinFunction{
    pub callback: InternalCallback,
    pub name: &'static str
}

impl Debug for BuiltinFunction{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("(builtin {})", self.name))
    }
}
