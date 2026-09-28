use crate::{expect_copy_type, expect_ref_type};
use crate::value::{EvalContext, EvalResult, EvalValue, ReferenceValue};
use crate::interpreter::eval_call_with_values;
use crate::value::list::List;
use crate::value::numeric::Numeric;
use crate::scope::ScopeRef;
use crate::stdlib::util::func;
use crate::value::builtin::{BuiltinFunction, BuiltInFunctionArgs};
use crate::value::error::EvalError;


fn list_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    let vals: Vec<EvalValue> = args.eval_all(scope)?;
    let list = List::from(vals.clone());
    Ok((EvalValue::Reference(ReferenceValue::List(list).to_rc()), EvalContext::none()))
}

fn wrap_opt_to_unit(v: Option<EvalValue>) -> EvalValue {
    match v {
        None => EvalValue::Unit,
        Some(v) => v
    }
}
fn nth_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    let (list_value, _) = args.try_pos(scope, 1)?.evaluated(scope)?;
    let list = expect_ref_type!(list_value, ReferenceValue::List(l) => l, scope)?;
    let (arg_value, _) = args.try_pos(scope, 0)?.evaluated(scope)?;
    let pos = expect_copy_type!(arg_value, EvalValue::Numeric(Numeric::Integer(pos)) => pos as usize, scope)?;
    Ok((wrap_opt_to_unit(list.get(pos)), EvalContext::none()))
}


fn car_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    let (arg_value, _ )  = args.try_pos(scope, 0)?.evaluated(scope)?;
    let list = expect_ref_type!(arg_value, ReferenceValue::List(v) => v, scope)?;
    Ok((wrap_opt_to_unit(list.head()), EvalContext::none()))
}

fn cdr_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    let (arg_value, _ )  = args.try_pos(scope, 0)?.evaluated(scope)?;
    let list = expect_ref_type!(arg_value, ReferenceValue::List(l) => l, scope)?;
    Ok((EvalValue::Reference(ReferenceValue::List(list.tail()).to_rc()), EvalContext::none()))
}


fn cons_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    let (arg_value, _ )  = args.try_pos(scope, 1)?.evaluated(scope)?;
    let list = expect_ref_type!(arg_value, ReferenceValue::List(l) => l, scope)?;
    let (con_value, _ )  = args.try_pos(scope, 0)?.evaluated(scope)?;
    Ok((EvalValue::Reference(ReferenceValue::List(list.prepended(con_value)).to_rc()), EvalContext::none()))
}




pub fn std_lists() -> Vec<BuiltinFunction> {
    vec![
        func("list", list_callback),
        func("car", car_callback),
        func("cdr", cdr_callback),
        func("cons", cons_callback),
        func("nth", nth_callback),
    ]
}
