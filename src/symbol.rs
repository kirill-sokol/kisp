//! Interned symbols.
//!
//! Every identifier is interned once, at parse time, into a `Sym`: a `u32` that
//! compares, hashes and copies for free. Scopes are keyed by it, so a variable
//! lookup is an integer compare (function frames) or a vector index (the global
//! scope) instead of hashing a `String`. Names are only needed for printing and
//! error messages. The interner is thread-local, like the rest of the `Rc`-based
//! interpreter.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

#[derive(Default)]
struct Interner {
    ids: HashMap<String, u32>,
    names: Vec<String>,
}

thread_local! {
    static INTERNER: RefCell<Interner> = RefCell::new(Interner::default());
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Sym(u32);

impl Sym {
    pub fn intern(name: &str) -> Sym {
        INTERNER.with(|i| {
            let mut i = i.borrow_mut();
            if let Some(&id) = i.ids.get(name) {
                return Sym(id);
            }
            let id = i.names.len() as u32;
            i.names.push(name.to_string());
            i.ids.insert(name.to_string(), id);
            Sym(id)
        })
    }

    pub fn id(self) -> u32 {
        self.0
    }

    /// The symbol's name (a copy).
    pub fn name(self) -> String {
        INTERNER.with(|i| i.borrow().names[self.0 as usize].clone())
    }
}

impl fmt::Display for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

impl fmt::Debug for Sym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sym({}#{})", self.name(), self.0)
    }
}

impl From<&str> for Sym {
    fn from(s: &str) -> Sym {
        Sym::intern(s)
    }
}

impl From<String> for Sym {
    fn from(s: String) -> Sym {
        Sym::intern(&s)
    }
}
