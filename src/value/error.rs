use crate::lexer::Cursor;
use crate::scope::ScopeRef;
use crate::stacktrace::StackTrace;

#[derive(Debug)]
pub enum EvalError{
    Other(String),
    UnknownSymbol(crate::symbol::Sym),
    CallingNonCallable,
    InvalidType,
    MissingArgument,
    NotImplemented,
    Reassignment,
    StackOverflow,
}

impl EvalError{
    pub fn trace(self, scope: &ScopeRef) -> ErrorContext {
        ErrorContext{
            error: self,
            stack_trace: Some(StackTrace::from_scope(scope))
        }
    }
}

#[derive(Debug)]
pub struct ErrorContext{
    error: EvalError,
    stack_trace: Option<StackTrace>
}
