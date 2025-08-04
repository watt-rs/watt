// imports
use crate::memory::gc::Gc;
use crate::values::Value;
use rustc_hash::FxHashMap;
use watt_common::address::Address;
use watt_common::{error, errors::Error};

/// Table
///
/// used by vm to set variables,
/// find variables, define variables,
/// delete variables, ...
///
#[derive(Clone, Debug)]
pub struct Table {
    /// this table fields list
    pub fields: FxHashMap<String, Value>,
    /// root table
    pub root: Option<Gc<Table>>,
    /// closure table
    pub closure: Option<Gc<Table>>,
}
/// Table implementation
impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Table {
    /// New table
    pub fn new() -> Table {
        Table {
            fields: FxHashMap::default(),
            root: None,
            closure: None,
        }
    }

    /// Checks variable exists in fields
    /// or closure
    ///
    pub unsafe fn exists(&self, name: &str) -> bool {
        if self.fields.contains_key(name) {
            true
        } else {
            match &self.closure {
                Some(closure) => closure.exists(name),
                None => false,
            }
        }
    }

    /// Finds variable in fields
    /// and closure
    ///
    /// raises error if not exists
    ///
    pub unsafe fn find(&self, address: &Address, name: &str) -> Value {
        if self.exists(name) {
            if self.fields.contains_key(name) {
                self.fields[name].clone()
            } else {
                match &self.closure {
                    Some(closure) => closure.find(address, name),
                    None => panic!(
                        "closure is None, but field {name} is exists. report this error to the devloper"
                    ),
                }
            }
        } else {
            error!(Error::own_text(
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
    pub fn define(&mut self, address: &Address, name: &str, value: Value) {
        if !self.fields.contains_key(name) {
            self.fields.insert(name.to_string(), value);
        } else {
            error!(Error::own_text(
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
    pub unsafe fn set(&mut self, address: &Address, name: &str, value: Value) {
        if self.fields.contains_key(name) {
            self.fields.insert(name.to_string(), value);
        } else if let Some(ref mut root) = self.root
            && root.has(name)
        {
            root.set(address, name, value);
        } else if let Some(ref mut closure) = self.closure
            && closure.has(name)
        {
            closure.set(address, name, value);
        } else {
            error!(Error::own_text(
                address.clone(),
                format!("{name} is not defined."),
                "check variable existence.",
            ));
        }
    }

    /// Sets variable in fields
    ///
    /// raises error if not defined
    ///
    pub unsafe fn set_local(&mut self, address: &Address, name: &str, value: Value) {
        if !self.fields.contains_key(name) {
            error!(Error::own_text(
                address.clone(),
                format!("{name} is not defined."),
                "you can define it, using := op.",
            ));
        }
        self.fields.insert(name.to_string(), value);
    }

    /// Checks variable exists in fields, closures or roots
    pub unsafe fn has(&self, name: &str) -> bool {
        if self.exists(name) {
            true
        } else if let Some(ref root) = self.root {
            root.has(name)
        } else {
            false
        }
    }

    /// Finds variable in fields
    /// closures, and roots
    ///
    /// raises error if not exists
    ///
    pub unsafe fn lookup(&self, address: &Address, name: &str) -> Value {
        if self.fields.contains_key(name) {
            self.fields[name].clone()
        } else if let Some(ref root) = self.root
            && root.has(name)
        {
            root.lookup(address, name)
        } else if let Some(ref closure) = self.closure
            && closure.has(name)
        {
            closure.find(address, name)
        } else {
            error!(Error::own_text(
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
    pub unsafe fn set_root(&mut self, new_root: Gc<Table>) {
        match &mut self.root {
            Some(root) => root.set_root(new_root.clone()),
            None => self.root = Some(new_root),
        }
    }

    /// Deletes last root from roots chain
    pub unsafe fn del_root(&mut self) {
        match &mut self.root {
            Some(root) => root.del_root(),
            None => self.root = None,
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
        if let Some(ref root) = self.root {
            root.print(indent + 1);
        }
        println!(
            "{space:spaces$}> parent: ",
            space = " ",
            spaces = indent * 2
        );
        if let Some(ref closure) = self.closure {
            closure.print(indent + 1);
        }
    }
}

/// Send & sync for future multi-threading.
unsafe impl Send for Table {}
unsafe impl Sync for Table {}
