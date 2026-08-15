use std::collections::HashMap;

use crate::{Error, Result};

pub struct Args {
    strings: HashMap<String, Option<String>>,
    alts: HashMap<String, Option<String>>,
    bools: HashMap<String, Option<bool>>,
}

impl Args {
    pub fn get_string(&mut self, name: &str) -> Result<Option<String>> {
        match self.strings.remove(name) {
            Some(o) => match o {
                Some(s) => Ok(Some(s)),
                None => Ok(None),
            },
            None => Err(Error::ArgNotAvailable),
        }
    }

    pub fn get_alt(&self, name: &str) -> Result<Option<&str>> {
        match self.alts.get(name) {
            Some(o) => match o {
                Some(s) => Ok(Some(s.as_str())),
                None => Ok(None),
            },
            None => Err(Error::ArgNotAvailable),
        }
    }

    pub fn get_bool(&self, name: &str) -> Result<Option<bool>> {
        match self.bools.get(name) {
            Some(o) => Ok(*o),
            None => Err(Error::ArgNotAvailable),
        }
    }

    pub(crate) fn new() -> Self {
        Args {
            strings: HashMap::new(),
            alts: HashMap::new(),
            bools: HashMap::new(),
        }
    }

    pub(crate) fn add_string(&mut self, name: String, val: Option<String>) {
        self.strings.insert(name, val);
    }

    pub(crate) fn add_alt(&mut self, name: String, val: Option<String>) {
        self.alts.insert(name, val);
    }

    pub(crate) fn add_bool(&mut self, name: String, val: Option<bool>) {
        self.bools.insert(name, val);
    }
}
