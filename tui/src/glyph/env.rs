//! Environment for Glyph: scope chain with parent links.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::value::{EvalError, EvalResult, Value};

#[derive(Debug, Clone)]
pub struct Env(Rc<RefCell<EnvNode>>);

#[derive(Debug)]
struct EnvNode {
    bindings: HashMap<String, Value>,
    parent: Option<Env>,
}

impl Env {
    pub fn new() -> Self {
        Env(Rc::new(RefCell::new(EnvNode {
            bindings: HashMap::new(),
            parent: None,
        })))
    }

    /// Create a child environment with a parent (lexical scope).
    pub fn extend(parent: &Env) -> Self {
        Env(Rc::new(RefCell::new(EnvNode {
            bindings: HashMap::new(),
            parent: Some(parent.clone()),
        })))
    }

    /// Bind a name to a value in the current scope.
    pub fn bind(&self, name: &str, value: Value) {
        self.0.borrow_mut().bindings.insert(name.to_string(), value);
    }

    /// Look up a name, searching up the parent chain.
    pub fn lookup(&self, name: &str) -> Option<Value> {
        let lower = name.to_lowercase();
        let node = self.0.borrow();
        match node.bindings.get(&lower) {
            Some(val) => Some(val.clone()),
            None => match &node.parent {
                Some(parent) => parent.lookup(&lower),
                None => None,
            },
        }
    }

    /// Set an existing binding in the innermost scope that has it.
    pub fn set(&self, name: &str, value: Value) -> EvalResult<()> {
        let lower = name.to_lowercase();
        {
            let node = self.0.borrow();
            if node.bindings.contains_key(&lower) {
                // found — drop borrow then mutate
            } else {
                return match &node.parent {
                    Some(parent) => parent.set(&lower, value),
                    None => Err(EvalError::UnboundSymbol(name.to_string())),
                };
            }
        }
        self.0.borrow_mut().bindings.insert(lower, value);
        Ok(())
    }

    /// Check if a name exists anywhere in the env chain.
    pub fn exists(&self, name: &str) -> bool {
        self.lookup(name).is_some()
    }

    /// Remove a binding from the current scope only (not parents).
    pub fn unbind(&self, name: &str) {
        let lower = name.to_lowercase();
        self.0.borrow_mut().bindings.remove(&lower);
    }
}
