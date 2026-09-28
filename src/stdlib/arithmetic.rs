use crate::expect_copy_type;
use crate::value::{EvalContext, EvalResult, EvalValue, ReferenceValue};
use crate::scope::ScopeRef;
use crate::value::numeric::Numeric;
use crate::stdlib::util::func;
use crate::value::builtin::{BuiltinFunction, BuiltInFunctionArgs};
use crate::value::error::{ErrorContext, EvalError};

fn function_with_reduction<T>(scope: &ScopeRef, args: BuiltInFunctionArgs<'_>, value_mapping: impl Fn(&EvalValue) -> Result<T, ErrorContext>, reduction: impl Fn(T, T) -> T) -> Result<T, ErrorContext> {
    // evaluate, map and reduce in one pass: no intermediate vectors, and the
    // first error terminates early
    let mut acc: Option<T> = None;
    for arg in args.iter() {
        let value = value_mapping(&arg.evaluated(scope)?.0)?;
        acc = Some(match acc {
            None => value,
            Some(a) => reduction(a, value),
        });
    }
    acc.map_or_else(|| Err(EvalError::MissingArgument.trace(scope)), Ok)
}


fn numeric_reduction(scope: &ScopeRef, args: BuiltInFunctionArgs<'_>, reduction: impl Fn(Numeric, Numeric) -> Numeric) -> EvalResult{
    let value_mapping =
        |value: &EvalValue| expect_copy_type!(value, EvalValue::Numeric(n) => n.clone(), scope);
    function_with_reduction(scope, args, value_mapping, reduction)
        .map(|i| (EvalValue::Numeric(i), EvalContext::none()))
}

fn addition_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    numeric_reduction(scope, args, |a, b| a+b)
}

fn subtraction_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    numeric_reduction(scope, args, |a, b| a-b)
}

fn multiplication_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    numeric_reduction(scope, args, |a, b| a*b)
}

fn division_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    numeric_reduction(scope, args, |a, b| a/b)
}



pub fn std_arithmetic() -> Vec<BuiltinFunction> {
    vec![
        func("+", addition_callback),
        func("-", subtraction_callback),
        func("*", multiplication_callback),
        func("/", division_callback),
    ]
}
