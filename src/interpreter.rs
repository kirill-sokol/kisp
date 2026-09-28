use std::cell::Cell;
use std::iter::Peekable;
use std::rc::Rc;
use std::slice::Iter;
use crate::ast::{PosExpression, SExpression};
use crate::symbol::Sym;
use crate::value::{EvalContext, EvalResult, EvalValue, ReferenceValue};


use crate::scope::{Scope, ScopeRef};
use crate::stdlib::std_lib_functions;
use crate::value::builtin::{BuiltinFunction, BuiltInFunctionArg, BuiltInFunctionArgs};
use crate::value::callable::{Callable, Function, Lambda, TailCall};
use crate::value::error::{ErrorContext, EvalError};

/// Maximum nesting of function/lambda calls before `EvalError::StackOverflow`.
/// Frames are now children of the *defining* scope (lexical scoping), so the
/// scope chain no longer reflects the call depth; this counter does.
pub const MAX_CALL_DEPTH: usize = 420;

thread_local! {
    static CALL_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// RAII guard: increments the call depth on creation, decrements on drop
/// (also when an error propagates with `?`).
struct CallDepthGuard;

impl CallDepthGuard {
    fn enter(scope: &ScopeRef) -> Result<CallDepthGuard, ErrorContext> {
        let depth = CALL_DEPTH.with(|d| d.get());
        if depth >= MAX_CALL_DEPTH {
            return Err(EvalError::StackOverflow.trace(scope));
        }
        CALL_DEPTH.with(|d| d.set(depth + 1));
        Ok(CallDepthGuard)
    }
}

impl Drop for CallDepthGuard {
    fn drop(&mut self) {
        CALL_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
    }
}

/// The function whose body is being evaluated in `scope`: the origin of the
/// nearest enclosing scope that has one. `[block]` scopes carry no origin of
/// their own, so a tail call inside a block body is still recognised.
fn enclosing_origin(scope: &ScopeRef) -> Option<Rc<ReferenceValue>> {
    let mut current = Some(scope.clone());
    while let Some(s) = current {
        if s.origin.is_some() {
            return s.origin.clone();
        }
        current = s.parent.clone();
    }
    None
}

fn env_scope() -> ScopeRef {
    let scope = Scope::new();
    scope.insert(Sym::intern("true"), EvalValue::True);
    for bi in std_lib_functions().into_iter() {
        scope.insert(Sym::intern(bi.name), EvalValue::Reference(ReferenceValue::CallableValue(Callable::Internal(bi)).to_rc()))
    }
    scope
}

pub fn eval(ast: &'_ PosExpression, provided_scope: Option<ScopeRef>) -> (EvalResult, ScopeRef) {
    let env = if let Some(provided) = provided_scope{
        provided
    }else{
        env_scope()
    };
    let res = match &ast.exp {
        //don't create a new scope!
        SExpression::Block(entries) => eval_block(EvalContext::none(), &env, entries, true),
        _ => eval_expression(EvalContext::none(), &env, ast)
    };
    (res, env)
}

pub(crate) fn eval_expression(ctx: EvalContext, scope: &ScopeRef, expression: &'_ PosExpression) -> EvalResult {
    match &expression.exp {
        // map_or_else: the error (with its stack trace of allocated strings)
        // must only be built when the lookup actually fails
        SExpression::Symbol(i) => scope.lookup(*i).map_or_else(
            || Err(EvalError::UnknownSymbol(*i).trace(scope)),
            |v| Ok((v, EvalContext::none()))
        ),
        SExpression::Number(i) => Ok((EvalValue::Numeric(i.clone()), EvalContext::none())),
        SExpression::List(expressions) => eval_list(ctx, scope, expressions),
        SExpression::Block(expressions) => eval_block(ctx, scope, expressions, false),
    }
}

fn populate_scope_with_args(scope: &ScopeRef, values: Vec<EvalValue>, arg_names: &Vec<Sym>) -> () {
    arg_names.iter()
        .zip(values)
        .for_each(
            |(ident, val)|
            scope.insert(*ident, val)
        );
}

fn is_tail_call(ctx: &EvalContext, origin: &Option<Rc<ReferenceValue>>, inside: &Option<Rc<ReferenceValue>>) -> bool {
    match (origin, inside) {
        (Some(o), Some(i) )if ctx.possible_tail =>  Rc::ptr_eq(o,i),
        (_, _) => false,
    }
}


/// `caller` is the scope the call expression is evaluated in (used to detect a
/// self tail call), `defining_scope` is the scope the function was created in
/// (the new frame becomes its child: lexical scoping, closures work).
pub fn wrap_tail_call(ctx: EvalContext, caller: &ScopeRef, defining_scope: &ScopeRef, passed_in: Vec<EvalValue>, arg_names: &Vec<Sym>, expression: &PosExpression, origin: Option<Rc<ReferenceValue>>) -> EvalResult {
    let tc_detected = is_tail_call(&ctx, &enclosing_origin(caller), &origin);
    if tc_detected{
        let tc: TailCall = TailCall{ function: origin.unwrap().clone(), args: passed_in };
        Ok((EvalValue::Reference(ReferenceValue::TailCallValue(tc).to_rc()), ctx))
    }else {
        eval_with_args(ctx, defining_scope, passed_in, arg_names, expression, origin)
    }
}

pub(crate) fn eval_with_args_flat(given_ctx: EvalContext, scope: &ScopeRef, passed_in: Vec<EvalValue>, arg_names: &Vec<Sym>, expression: &PosExpression, _origin: Option<Rc<ReferenceValue>>) -> EvalResult {
    populate_scope_with_args(&scope, passed_in, arg_names);
    let (mut res, mut res_ctx) = eval_expression(
        EvalContext{possible_tail: true}, //there we go, tail recursion
        &scope,
        expression
    )?;
    while let EvalValue::Reference(r)= &res {
        match r.as_ref(){
            ReferenceValue::TailCallValue(tc) => {
                scope.clear();
                populate_scope_with_args(&scope, tc.args.clone(), arg_names);
                (res, res_ctx) = eval_expression(EvalContext{possible_tail: true}, scope, expression)?;
            },
            _=> break
        }

    }
    Ok((res, EvalContext{possible_tail: given_ctx.possible_tail && res_ctx.possible_tail}))
}

pub(crate) fn eval_with_args(ctx: EvalContext, scope: &ScopeRef, passed_in: Vec<EvalValue>, arg_names: &Vec<Sym>, expression: &PosExpression, origin: Option<Rc<ReferenceValue>>) -> EvalResult {
    let _depth = CallDepthGuard::enter(scope)?;
    let func_scope = scope.enter(origin.clone())?;
    let result = eval_with_args_flat(ctx, &func_scope, passed_in, arg_names, expression, origin);
    Scope::release(func_scope);
    result
}


fn eval_all(_ctx: EvalContext, scope: &ScopeRef, exps: & [PosExpression]) -> Result<Vec<EvalValue>, ErrorContext> {
    exps.iter()
        .map(|exp| eval_expression(EvalContext::none(), scope, exp).map(|t| t.0))
        .collect()
}

pub(crate) fn eval_call_with_values(ctx: EvalContext, scope: &ScopeRef, callable: &Callable, args: Vec<EvalValue>, origin: Option<Rc<ReferenceValue>>) -> EvalResult {
    match callable {
        Callable::Internal(BuiltinFunction{callback,..}) => callback(
            scope,
            ctx,
            BuiltInFunctionArgs::from(args),
        ),
        Callable::Function(func) =>
            wrap_tail_call(ctx, scope, &func.in_scope, args, &func.arguments, &func.body, origin),
        Callable::Lambda(lam) => eval_with_args(EvalContext::none(), &lam.in_scope, args, &lam.arguments, &lam.body, None),
    }
}


pub(crate) fn eval_callable(ctx: EvalContext, scope: &ScopeRef, callable: &Callable, args: &'_ [PosExpression], origin: Option<Rc<ReferenceValue>>) -> EvalResult {
    match callable {
        Callable::Internal(bi) => {
            // arguments are handed over unevaluated, borrowed from the AST
            (bi.callback)(scope, ctx, BuiltInFunctionArgs::Expressions(args))
        },
        Callable::Function(Function{arguments: _, body: _,..}) =>
            eval_call_with_values(ctx, scope, callable, eval_all(EvalContext::none(), scope, args)?, origin),

        Callable::Lambda(Lambda{arguments: _, body: _, ..}) =>
            eval_call_with_values(ctx, scope, callable, eval_all(EvalContext::none(), scope, args)?, origin),
    }
}

pub(crate) fn eval_list(ctx: EvalContext, scope: &ScopeRef, expressions: &'_ Vec<PosExpression>) -> EvalResult {

    if expressions.is_empty(){
        return Ok((EvalValue::Unit, EvalContext::none())); //not sure how well this notation is, but whatever
    }
    let (head_value, _) = eval_expression(EvalContext::none(), scope, expressions.first().unwrap())?;
    let tail = &expressions[1..];
    let (r, callable) = match &head_value {
        EvalValue::Reference(r) => {
            match r.as_ref(){
                ReferenceValue::CallableValue(c) => Ok((r.clone(),c)),
                _ => Err(EvalError::CallingNonCallable.trace(scope))
            }
        }
        _ => Err(EvalError::CallingNonCallable.trace(scope))
    }?;
    eval_callable(ctx, scope, callable, tail, Some(r.clone()))
}

fn eval_block_iter(ctx: EvalContext, scope: &ScopeRef, iterator: &mut Peekable<Iter<'_, PosExpression>>, last: (EvalValue, EvalContext)) -> EvalResult {
    match iterator.next() {
        None => Ok(last),
        Some(exp) =>
            eval_expression(
                //last element could be a tail
                EvalContext{possible_tail: iterator.peek().is_none() && ctx.possible_tail},
                scope,
                exp
            ).and_then(
                |v|
                    // keep the block's own tail context: only the *last* element
                    // may be a tail call, which the check above decides per element
                    eval_block_iter(ctx, scope, iterator, v)
            )
    }
}

pub(crate) fn eval_block(ctx: EvalContext, scope: &ScopeRef, expressions: &'_ Vec<PosExpression>, flat: bool) -> EvalResult {
    let block_scope = scope.enter(None)?;
    eval_block_iter(ctx, if flat {scope} else {&block_scope}, &mut expressions.iter().peekable(), (EvalValue::Unit, EvalContext::none()))
}
