use std::cell::RefCell;
use std::rc::Rc;
use crate::symbol::Sym;
use crate::value::{EvalValue, ReferenceValue};
use crate::value::error::ErrorContext;

pub type ScopeRef = Rc<Scope>;

thread_local! {
    static FRAME_POOL: RefCell<Vec<Rc<Scope>>> = const { RefCell::new(Vec::new()) };
}

/// The bindings of one scope. The global scope holds every builtin and every
/// top-level `let`/`fn`, so it is a table indexed by symbol id; a function frame
/// holds a handful of arguments, so a linear scan over ids is faster than a map
/// and needs one small allocation.
#[derive(Debug)]
enum Entries {
    Table(Vec<Option<EvalValue>>),
    Frame(Vec<(Sym, EvalValue)>),
}

#[derive(Debug)]
pub struct Scope {
    pub origin: Option<Rc<ReferenceValue>>, //TODO: it should really only expect function value
    pub depth: usize,
    pub parent: Option<ScopeRef>,
    entries: RefCell<Entries>,
    vararg: Vec<EvalValue>
}

impl Scope {
    pub fn new() -> Rc<Self> {
        Rc::new(Scope{origin: None,depth:0,parent: None, entries: RefCell::new(Entries::Table(Vec::new())), vararg: Default::default()})
    }

    pub fn enter(self: &Rc<Self>, origin: Option<Rc<ReferenceValue>>) -> Result<Rc<Self>, ErrorContext> {
        self.enter_with_vararg(vec![], origin)
    }

    pub fn vararg<'scope>(self: &'scope Rc<Self>) -> &'scope Vec<EvalValue> {
        &self.vararg
    }

    pub fn enter_with_vararg(self: &Rc<Self>, vararg: Vec<EvalValue>, origin: Option<Rc<ReferenceValue>>) -> Result<Rc<Self>, ErrorContext> {
        // Recursion is bounded by interpreter::MAX_CALL_DEPTH (a call counter),
        // not by the chain length: with lexical scoping a frame's parent is the
        // defining scope, so the chain stays short however deep the recursion.
        // Frames are recycled (see `release`), so a call in steady state
        // allocates neither the Rc nor the entries vector.
        let recycled = FRAME_POOL.with(|p| p.borrow_mut().pop());
        if let Some(mut rc) = recycled {
            if let Some(scope) = Rc::get_mut(&mut rc) {
                scope.origin = origin;
                scope.depth = self.depth + 1;
                scope.parent = Some(self.clone());
                scope.vararg = vararg;
                return Ok(rc);
            }
        }
        Ok(
            Rc::new(Self{origin, depth: self.depth+1, parent: Some(self.clone()), entries: RefCell::new(Entries::Frame(Vec::with_capacity(4))), vararg})
        )
    }

    /// Give a finished frame back to the pool, unless a closure captured it.
    pub fn release(mut rc: Rc<Self>) {
        if let Some(scope) = Rc::get_mut(&mut rc) {
            if !matches!(&*scope.entries.borrow(), Entries::Frame(_)) {
                return;
            }
            scope.clear();
            scope.origin = None;
            scope.parent = None;
            scope.vararg.clear();
            FRAME_POOL.with(|p| {
                let mut p = p.borrow_mut();
                if p.len() < 64 {
                    p.push(rc);
                }
            });
        }
    }

    #[inline]
    fn lookup_local(&self, sym: Sym) -> Option<EvalValue> {
        match &*self.entries.borrow() {
            Entries::Table(t) => t.get(sym.id() as usize).and_then(|v| v.clone()),
            Entries::Frame(f) => f.iter().rev().find(|(s, _)| *s == sym).map(|(_, v)| v.clone()),
        }
    }

    pub fn lookup(&self, sym: Sym) -> Option<EvalValue> {
        let mut scope = self;
        loop {
            if let Some(v) = scope.lookup_local(sym) {
                return Some(v);
            }
            match &scope.parent {
                Some(p) => scope = p,
                None => return None,
            }
        }
    }

    pub fn clear(&self) -> (){
        match &mut *self.entries.borrow_mut() {
            Entries::Table(t) => t.clear(),
            Entries::Frame(f) => f.clear(),
        }
    }

    pub fn insert(&self, sym: Sym, value: EvalValue) -> () {
        match &mut *self.entries.borrow_mut() {
            Entries::Table(t) => {
                let i = sym.id() as usize;
                if i >= t.len() {
                    t.resize(i + 1, None);
                }
                t[i] = Some(value);
            }
            Entries::Frame(f) => {
                if let Some(slot) = f.iter_mut().find(|(s, _)| *s == sym) {
                    slot.1 = value;
                } else {
                    f.push((sym, value));
                }
            }
        }
    }
}
