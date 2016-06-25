//! Helper functions and core types required for decoding configuration from `config::Value`.
//!
//! Note that at this level:
//!
//! - We don't know what kind of configuration it is;
//! - We don't know where it comes from (TOML yes, but it is not dependent on that);
//! - We don't assume it will be human who reads it.

use std::fmt;
use std::result;
use Value;
use Table;

#[derive(Debug)]
pub struct Property {
    pub name: String,
    pub desc: String,
}

#[derive(Debug)]
pub enum Error {
    ExpectedTable {
        desc: String,
    },
    ExpectedString {
        desc: String,
    },
    ExpectedInteger {
        desc: String,
    },
    ExpectedFloat {
        desc: String,
    },
    ExpectedBool {
        desc: String,
    },
    ExpectedDatetime {
        desc: String,
    },
    ExpectedSlice {
        desc: String,
    },
    ExpectedOneOfTypes {
        found_type: String,
        possible_list: Vec<String>,
    },
    ExpectedProperty(Property),
    ExpectedProperties(Vec<Property>),
    ExpectedOneOfProperties(Vec<Property>),
    IncorrectValue {
        explanation: Option<String>,
        value: Value,
        possible_list: Vec<Value>,
    },
}

impl Error {
    pub fn at(self, path: String) -> At<Error> {
        At {
            error: self,
            path: path,
        }
    }
}

#[derive(Debug)]
pub struct At<E: fmt::Debug> {
    pub error: E,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Path<'a> {
    path: Vec<&'a str>,
    value: &'a Value,
    desc: &'a str,
}

/// Path class here encapsulates decoding a value at specific path.
///
/// Additionaly this value contains description that will be used in errors.
///
/// There are methods like `table_property` to drill down deeper into this value and
/// construct the documentation at the same time.
///
/// And in case the error occurs, the result will contain perfect information of what and
/// where happened.
impl<'a> Path<'a> {
    /// Construct root value with specified description.
    pub fn new<'r>(value: &'r Value, desc: &'r str) -> Path<'r> {
        Path {
            path: vec![],
            value: value,
            desc: desc,
        }
    }

    /// Construct a value at specified path.
    pub fn new_at<'r>(value: &'r Value, path: Vec<&'r str>, desc: &'r str) -> Path<'r> {
        Path {
            path: path,
            value: value,
            desc: desc,
        }
    }

    /// Clone into a new Path with specified value.
    pub fn clone_with(&'a self, value: &'a Value) -> Path<'a> {
        Path::<'a> {
            value: value,
            path: self.path.clone(),
            desc: self.desc,
        }
    }

    /// Return path to this configuration.
    pub fn components(&self) -> &[&str] {
        &self.path
    }

    /// Return raw value of this configuration.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Return description of the configuration at this path.
    pub fn description(&self) -> &str {
        &self.desc
    }

    /// Join decode path component that must be a property of this table value.
    ///
    /// Will return error if the value is not a table, or the table does not have
    /// specified property.
    pub fn table_property(&'a self,
                          property_name: &'a str,
                          property_desc: &'a str)
                          -> Result<Path<'a>> {
        let mut path = self.path.clone();
        path.push(property_name);
        Ok(Path::<'a> {
            value: match try!(self.as_table()).get(property_name) {
                Some(value) => value,
                None => {
                    return Err(Error::ExpectedProperty(Property {
                            name: property_name.to_string(),
                            desc: property_desc.to_string(),
                        })
                        .at(Self::path_as_string(&path)))
                }
            },
            path: path,
            desc: property_desc,
        })
    }

    /// Join decode path component and use specified value as if it was the child.
    pub fn join(&'a self,
                value: &'a Value,
                property_name: &'a str,
                property_desc: &'a str)
                -> Path<'a> {
        let mut path = self.path.clone();
        path.push(property_name);
        Path::<'a> {
            value: value,
            path: path,
            desc: property_desc,
        }
    }

    pub fn as_str(&self) -> Result<&str> {
        self.value
            .as_str()
            .ok_or_else(|| Error::ExpectedTable { desc: self.desc.into() }.at(self.to_string()))
    }

    pub fn as_integer(&self) -> Result<i64> {
        self.value
            .as_integer()
            .ok_or_else(|| Error::ExpectedInteger { desc: self.desc.into() }.at(self.to_string()))
    }

    pub fn as_float(&self) -> Result<f64> {
        self.value
            .as_float()
            .ok_or_else(|| Error::ExpectedFloat { desc: self.desc.into() }.at(self.to_string()))
    }

    pub fn as_bool(&self) -> Result<bool> {
        self.value
            .as_bool()
            .ok_or_else(|| Error::ExpectedBool { desc: self.desc.into() }.at(self.to_string()))
    }

    pub fn as_datetime(&self) -> Result<&str> {
        self.value
            .as_datetime()
            .ok_or_else(|| Error::ExpectedDatetime { desc: self.desc.into() }.at(self.to_string()))
    }

    pub fn as_slice(&self) -> Result<&[Value]> {
        self.value
            .as_slice()
            .ok_or_else(|| Error::ExpectedSlice { desc: self.desc.into() }.at(self.to_string()))
    }

    pub fn as_table(&self) -> Result<&Table> {
        self.value
            .as_table()
            .ok_or_else(|| Error::ExpectedTable { desc: self.desc.into() }.at(self.to_string()))
    }

    fn path_as_string(path: &Vec<&str>) -> String {
        let mut result = String::new();
        let mut first = true;
        for v in path {
            if first {
                result.push_str(v);
                first = false;
                continue;
            }
            result.push('.');
            result.push_str(v);
        }
        result
    }
}

impl<'a> fmt::Display for Path<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Self::path_as_string(&self.path).fmt(f)
    }
}

pub type Result<T> = result::Result<T, At<Error>>;
