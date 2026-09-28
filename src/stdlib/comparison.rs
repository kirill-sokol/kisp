use crate::expect_copy_type;
use crate::value::{EvalContext, EvalResult, EvalValue, ReferenceValue};
use crate::scope::ScopeRef;
use crate::value::numeric::Numeric;
use crate::stdlib::util::func;
use crate::value::builtin::{BuiltinFunction, BuiltInFunctionArgs};
use crate::value::error::EvalError;

// < > = >= <= !=

//TODO: create some quick type conversion macros
fn comparison_reduction(scope: &ScopeRef, args: BuiltInFunctionArgs<'_>, operation: fn(Numeric, Numeric) -> bool) -> EvalResult {
    let head_arg = args.try_pos(scope, 0)?.evaluated(scope)?.0;
    let head = expect_copy_type!(head_arg, EvalValue::Numeric(n) => n, scope)?;
    //expect_copy_type!(args.try_pos(scope, 0)?.evaluated(scope)?.0, Numeric(n) => i.clone(), None)?;
    for v in args.iter_from(1) {
        let r_arg = v.evaluated(scope)?.0;
        let r_value = expect_copy_type!(r_arg, EvalValue::Numeric(n) => n, scope)?;
        if !operation(head.clone(), r_value){
            return Ok((EvalValue::Unit, EvalContext::none())); //early return, don't even evaluate the rest
        }
    }
    Ok((EvalValue::True, EvalContext::none()))
}

fn gt_callback(scope: &ScopeRef, _ctx: EvalContext,  args: BuiltInFunctionArgs<'_>) -> EvalResult {
    comparison_reduction(scope, args, |h, v| h>v)
}

fn gt_eq_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    comparison_reduction(scope, args, |h, v| h>=v)
}

fn lt_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    comparison_reduction(scope, args, |h, v| h<v)
}

fn lt_eq_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    comparison_reduction(scope, args, |h, v| h<=v)
}

fn eq_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    comparison_reduction(scope, args, |h, v| h==v)
}

fn neq_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    comparison_reduction(scope, args, |h, v| h!=v)
}

pub fn std_comparison() -> Vec<BuiltinFunction> {
    vec![

        func(">", gt_callback),
        func(">=", gt_eq_callback),
        func("<", lt_callback),
        func("<=", lt_eq_callback),
        func("=", eq_callback),
        func("!=", neq_callback),
    ]
}
