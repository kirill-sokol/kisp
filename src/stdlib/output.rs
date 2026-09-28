use crate::value::{EvalContext, EvalResult, EvalValue, ReferenceValue};
use crate::scope::ScopeRef;
use crate::stdlib::util::{func};
use crate::value::builtin::{BuiltinFunction, BuiltInFunctionArgs};

fn print_callback(scope: &ScopeRef, _ctx: EvalContext, args: BuiltInFunctionArgs<'_>) -> EvalResult {
    let vals = args.eval_all(scope)?;
    let string = vals.iter()
        .map(|v|v.to_string())
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", string);
    Ok((EvalValue::Unit, EvalContext::none()))
}

pub fn std_output() -> Vec<BuiltinFunction> {
    vec![

        func("print", print_callback)
    ]
}
