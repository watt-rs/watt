// imports
use crate::errors::errors::Error;
use crate::lexer::address::Address;
use crate::vm::memory::memory;
use crate::vm::values::Value;
use rustc_hash::FxHashMap;
use std::collections::HashSet;

/// Table
///
/// used by vm to set variables,
/// find variables, define variables,
/// delete variables, ...
///
#[derive(Clone)]
pub struct Table {
    /// this table fields list
    pub fields: FxHashMap<String, Value>,
    ///
    /// root table, previous lexical table
    /// for example:
    /// ```
    /// if a { // table one         
    ///   if b { // table two, root: table one
    ///   }
    /// }
    /// ```
    ///
    pub root: *mut Table,
    /// parent table, previous chunk run table
    pub parent: *mut Table,
    /// closure table
    pub closure: *mut Table,
    /// captures amount
    pub captures: usize,
}
/// Table implementation
impl Table {
    /// New table
    pub fn new() -> Table {
        Table {
            fields: FxHashMap::default(),
            root: std::ptr::null_mut(),
            parent: std::ptr::null_mut(),
            closure: std::ptr::null_mut(),
            captures: 0,
        }
    }

    /// Checks variable exists in fields
    /// or closure
    ///
    pub unsafe fn exists(&self, name: &str) -> bool {
        if self.fields.contains_key(name) {
            true
        } else {
            !self.closure.is_null() && (*self.closure).exists(name)
        }
    }

    /// Finds variable in fields
    /// and closure
    ///
    /// raises error if not exists
    ///
    pub unsafe fn find(&self, address: &Address, name: &str) -> Result<Value, Error> {
        if self.exists(name) {
            if self.fields.contains_key(name) {
                Ok(self.fields[name])
            } else {
                Ok((*self.closure).find(address, name)?)
            }
        } else {
            Err(Error::own_text(
                address.clone(),
                format!("{name} is not defined."),
                "check variable existence.",
            ))
        }
    }

    /// Defines variable in fields
    ///
    /// raises error if already defined
    ///
    pub fn define(&mut self, address: &Address, name: &str, value: Value) -> Result<(), Error> {
        if !self.fields.contains_key(name) {
            self.fields.insert(name.to_string(), value);
            Ok(())
        } else {
            Err(Error::own_text(
                address.clone(),
                format!("{name} is already defined."),
                "you can rename variable.",
            ))
        }
    }

    /// Sets variable in fields or roots
    ///
    /// raises error if not defined
    ///
    pub unsafe fn set(&mut self, address: Address, name: &str, value: Value) -> Result<(), Error> {
        if self.fields.contains_key(name) {
            self.fields.insert(name.to_string(), value);
        } else if !self.root.is_null() && (*self.root).has(name) {
            (*self.root).set(address, name, value)?;
        } else if !self.closure.is_null() && (*self.closure).exists(name) {
            (*self.closure).set(address, name, value)?;
        } else {
            return Err(Error::own_text(
                address.clone(),
                format!("{name} is not defined."),
                "check variable existence.",
            ));
        }
        Ok(())
    }

    /// Sets variable in fields
    ///
    /// raises error if not defined
    ///
    pub unsafe fn set_local(
        &mut self,
        address: &Address,
        name: &str,
        value: Value,
    ) -> Result<(), Error> {
        if !self.fields.contains_key(name) {
            return Err(Error::own_text(
                address.clone(),
                format!("{name} is not defined."),
                "you can define it, using := op.",
            ));
        }
        self.fields.insert(name.to_string(), value);
        Ok(())
    }

    /// Checks variable exists in fields, closures or roots
    pub unsafe fn has(&mut self, name: &str) -> bool {
        if self.exists(name) {
            true
        } else if !self.root.is_null() {
            (*self.root).has(name)
        } else {
            false
        }
    }

    /// Finds variable in fields
    /// closures, and roots
    ///
    /// raises error if not exists
    ///
    pub unsafe fn lookup(&mut self, address: &Address, name: &str) -> Result<Value, Error> {
        if self.fields.contains_key(name) {
            Ok(self.fields[name])
        } else if !self.root.is_null() && (*self.root).has(name) {
            (*self.root).lookup(address, name)
        } else if !self.closure.is_null() && (*self.closure).exists(name) {
            (*self.closure).find(address, name)
        } else {
            Err(Error::own_text(
                address.clone(),
                format!("{name} is not defined."),
                "check variable existence.",
            ))
        }
    }

    /// Sets root
    ///
    /// if root already exists, set root to root, and if root's root is already
    /// exists, set root to root's root, ...
    ///
    pub unsafe fn set_root(&mut self, root: *mut Table) {
        if self.root.is_null() {
            self.root = root
        } else {
            (*self.root).set_root(root);
        }
    }

    /// Deletes last root from roots chain
    #[allow(unused)]
    pub unsafe fn del_root(&mut self) {
        if !self.root.is_null() {
            if (*self.root).root.is_null() {
                self.root = std::ptr::null_mut();
            } else {
                (*self.root).del_root()
            }
        }
    }

    /// Frees all field values
    pub unsafe fn free_fields(&self) {
        // to free list
        let to_free: HashSet<&Value> = self.fields.values().collect();

        // freeing
        for val in to_free {
            match *val {
                Value::Fn(f) => {
                    if !f.is_null() {
                        memory::free_value(f);
                    }
                }
                Value::Instance(i) => {
                    if !i.is_null() {
                        memory::free_value(i);
                    }
                }
                Value::String(s) => {
                    if !s.is_null() {
                        memory::free_const_value(s);
                    }
                }
                Value::Native(n) => {
                    if !n.is_null() {
                        memory::free_value(n);
                    }
                }
                Value::Unit(u) => {
                    if !u.is_null() {
                        memory::free_value(u);
                    }
                }
                Value::List(l) => {
                    if !l.is_null() {
                        memory::free_value(l);
                    }
                }
                Value::Type(t) => {
                    if !t.is_null() {
                        memory::free_value(t);
                    }
                }
                Value::Trait(t) => {
                    if !t.is_null() {
                        memory::free_value(t);
                    }
                }
                _ => {}
            }
        }
    }

    /// Prints table tree
    #[allow(unused)]
    pub unsafe fn print(&self, indent: usize) {
        println!("{space:spaces$}Table:", space = " ", spaces = indent * 2);
        println!(
            "{space:spaces$}> [{:?}]",
            self.fields.keys(),
            space = " ",
            spaces = indent * 2
        );
        println!("{space:spaces$}> root: ", space = " ", spaces = indent * 2);
        if !self.root.is_null() {
            (*self.root).print(indent + 1);
        }
        println!(
            "{space:spaces$}> parent: ",
            space = " ",
            spaces = indent * 2
        );
        if !self.parent.is_null() {
            (*self.parent).print(indent + 1);
        }
        println!(
            "{space:spaces$}> closure: ",
            space = " ",
            spaces = indent * 2
        );
        if !self.closure.is_null() {
            (*self.closure).print(indent + 1);
        }
    }
}

/// Send & sync for future multi-threading.
unsafe impl Send for Table {}
unsafe impl Sync for Table {}
