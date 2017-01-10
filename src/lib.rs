//! A library to check the values from a submitted form or a query string.
//!
//! This library provides a validator, with the hope of helping Web
//! developers check user-submitted values in an easy and declarative way.
//!
//! # Examples
//!
//! ```
//! # use form_checker::{Validator, Checker, Rule, Str, I64};
//! // Prepare params, this is just for illustrating. Usually, we get
//! // params through decoding a URL-encoded string into a
//! // HashMap<String, Vec<String>>.
//! let mut params = std::collections::HashMap::new();
//! params.insert("name".to_string(), vec!["bob".to_string()]);
//! params.insert("age".to_string(), vec!["20".to_string()]);
//!
//! // Make a new Validator.
//! let mut validator = Validator::new();
//! // Add Checkers to Validator.
//! validator
//!     .check(Checker::new("name", "姓名", Str)
//!            .meet(Rule::Max(5))
//!            .meet(Rule::Min(2)))
//!     .check(Checker::new("age", "年龄", I64)
//!            .meet(Rule::Max(100))
//!            .meet(Rule::Min(18)));
//! // Validate it!
//! validator.validate(&params);
//! // Decide whether it is valid.
//! assert!(validator.is_valid());
//! // Show me the valid data, assuming it is valid.
//! assert_eq!(validator.get_required("name").as_str().unwrap(), "bob".to_string());
//! assert_eq!(validator.get_required("age").as_i64().unwrap(), 20);
//! ```

extern crate regex;

use std::fmt;
use std::collections::HashMap;
use regex::Regex;

pub struct Validator<T: IntoMessage=()> {
    pub checkers: Vec<Box<Checkable>>,
    pub valid_data: HashMap<String, Option<Vec<FieldValue>>>,
    pub invalid_messages: HashMap<String, String>,
    pub message: T,
}

pub trait Checkable {
    fn check(&self, params: &HashMap<String, Vec<String>>) -> Result<Option<Vec<FieldValue>>, Message>;

    fn get_name(&self) -> String;
}

impl Validator<()> {
    pub fn new() -> Validator<()> {
        Validator::with_message(())
    }
}

impl<T: IntoMessage> Validator<T> {
    pub fn with_message(message: T) -> Validator<T> {
        Validator {
            checkers: Vec::new(),
            valid_data: HashMap::new(),
            invalid_messages: HashMap::new(),
            message: message,
        }
    }

    pub fn check<U: Checkable + 'static>(&mut self, checker: U) -> &mut Validator<T> {
        self.checkers.push(Box::new(checker));
        self
    }

    pub fn validate(&mut self, params: &HashMap<String, Vec<String>>) {
        for checker in &self.checkers {
            match checker.check(params) {
                Ok(v) => {
                    self.valid_data.insert(checker.get_name().clone(), v);
                },
                Err(msg) => {
                    self.invalid_messages.insert(checker.get_name().clone(), msg.format_message(&self.message));
                },
            }
        }
    }

    pub fn get_required(&self, name: &str) -> FieldValue {
        self.valid_data.get(name).unwrap().as_ref().unwrap()[0].clone()
    }

    pub fn get_optional(&self, name: &str) -> Option<FieldValue> {
        match self.valid_data.get(name).unwrap().as_ref() {
            Some(v) => Some(v[0].clone()),
            None => None,
        }
    }

    pub fn get_required_multiple(&self, name: &str) -> Vec<FieldValue> {
        self.valid_data.get(name).unwrap().as_ref().unwrap().clone()
     }

    pub fn get_optional_multiple(&self, name: &str) -> Option<Vec<FieldValue>> {
        self.valid_data.get(name).unwrap().clone()
    }

    pub fn is_valid(&self) -> bool {
        self.valid_data.len() == self.checkers.len()
    }

    pub fn get_error(&self, name: &str) -> String {
        self.invalid_messages.get(name).unwrap().clone()
    }

    pub fn reset(&mut self) {
        self.valid_data.clear();
        self.invalid_messages.clear();
    }
}

enum MessageKey {
    Max,
    Min,
    MaxLen,
    MinLen,
    Blank,
    Format,
    Custom,
}

pub trait IntoMessage {
    fn max(&self, name: &str, value: &str) -> String {
        format!("{name}不能大于{value}", name=name, value=value)
    }

    fn min(&self, name: &str, value: &str) -> String {
        format!("{name}不能小于{value}", name=name, value=value)
    }

    fn max_len(&self, name: &str, value: &str) -> String {
        format!("{name}长度不能大于{value}", name=name, value=value)
    }

    fn min_len(&self, name: &str, value: &str) -> String {
        format!("{name}长度不能小于{value}", name=name, value=value)
    }

    fn blank(&self, name: &str) -> String {
        format!("{name}不能为空", name=name)
    }

    fn format(&self, name: &str) -> String {
        format!("{name}格式不正确", name=name)
    }
}

impl IntoMessage for () {
}

pub struct Message {
    key: MessageKey,
    values: HashMap<String, String>,
}

impl Message {
    fn format_message<T: IntoMessage>(&self, m: &T) -> String {
        match self.key {
            MessageKey::Max => m.max(self.values.get("name").unwrap(), self.values.get("value").unwrap()),
            MessageKey::Min => m.min(self.values.get("name").unwrap(), self.values.get("value").unwrap()),
            MessageKey::MaxLen => m.max_len(self.values.get("name").unwrap(), self.values.get("value").unwrap()),
            MessageKey::MinLen => m.min_len(self.values.get("name").unwrap(), self.values.get("value").unwrap()),
            MessageKey::Blank => m.blank(self.values.get("name").unwrap()),
            MessageKey::Format => m.format(self.values.get("name").unwrap()),
            MessageKey::Custom => self.values.get("value").unwrap().clone(),
        }
    }
}

pub enum CheckerOption {
    Optional(bool),
    Multiple(bool),
}

pub struct Checker<T: FieldType> {
    field_name: String,
    field_title: String,
    field_type: T,
    rules: Vec<Rule>,
    optional: bool,
    multiple: bool,
}

impl<T: FieldType> Checkable for Checker<T> {
    fn check(&self, params: &HashMap<String, Vec<String>>) -> Result<Option<Vec<FieldValue>>, Message> {
        let values = params.get(&self.field_name);

        if values.is_none() {
            if !self.optional {
                return Err(Message {
                    key: MessageKey::Blank,
                    values: {
                        let mut v = HashMap::new();
                        v.insert("name".to_string(), self.field_title.clone());
                        v
                    }});
            }

            return Ok(None)
        }

        let values = values.unwrap();

        let mut valid_values = Vec::new();

        if self.multiple {
            for value in values {
                match self.check_value(value) {
                    Ok(v) => valid_values.push(v),
                    Err(msg) => { return Err(msg); }
                }
            }
        } else {
            if values.len() < 1 {
                if !self.optional {
                    return Err(Message {
                        key: MessageKey::Blank,
                        values: {
                            let mut v = HashMap::new();
                            v.insert("name".to_string(), self.field_title.clone());
                            v
                        }});
                }

                return Ok(None)
            }

            match self.check_value(&values[0]) {
                Ok(v) => valid_values.push(v),
                Err(msg) => { return Err(msg); }
            }
        }

        Ok(Some(valid_values))

    }

    fn get_name(&self) -> String {
        self.field_name.clone()
    }

}

impl<T: FieldType> Checker<T> {
    pub fn new(field_name: &str, field_title: &str, field_type: T) -> Checker<T> {
        Checker {
            field_name: field_name.to_string(),
            field_title: field_title.to_string(),
            field_type: field_type,
            rules: Vec::new(),
            optional: false,
            multiple: false,
        }
    }

    fn check_value(&self, value: &str) -> Result<FieldValue, Message> {
        let field_value = try!(self.field_type.from_str(&self.field_title, value));
        for rule in &self.rules {
            if let Err(msg) = field_value.match_rule(&self.field_title, value, rule) {
                return Err(msg);
            }
        }
        Ok(field_value)
    }

    pub fn meet(mut self, rule: Rule) -> Checker<T> {
        self.rules.push(rule);
        self
    }

    pub fn set(mut self, option: CheckerOption) -> Checker<T> {
        match option {
            CheckerOption::Optional(optional) => {
                self.optional = optional;
            },
            CheckerOption::Multiple(multiple) => {
                self.multiple = multiple;
            }
        }
        self
    }

}

pub enum Rule {
    Max(i64),
    Min(i64),
    Format(&'static str),
    Lambda(Box<Fn(FieldValue) -> bool>, Option<Box<Fn(&str, &str) -> String>>)
}

pub trait FieldType {
    fn from_str(&self, field_title: &str, value: &str) -> Result<FieldValue, Message>;
}

#[derive(Clone)]
pub enum FieldValue {
    Str(String),
    I64(i64),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FieldValue::Str(ref s) => { write!(f, "{}", s) },
            FieldValue::I64(i) => { write!(f, "{}", i.to_string()) }
        }
    }
}

impl FieldValue {
    pub fn as_str(&self) -> Option<String> {
        match *self {
            FieldValue::Str(ref s) => Some(s.clone()),
            _ => None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            FieldValue::I64(i) => Some(i),
            _ => None
        }
    }

    fn match_rule(&self, field_title: &str, raw: &str, rule: &Rule) -> Result<(), Message> {
        match *rule {
            Rule::Lambda(ref f, ref err_handler) => {
                if !f(self.clone()) {
                    match *err_handler {
                        Some(ref handler) => {
                            return Err(Message {
                                key: MessageKey::Custom,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("value".to_string(), handler(field_title, raw));
                                    v
                                }
                            })
                        },
                        None => {
                            return Err(Message {
                                key: MessageKey::Format,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), field_title.to_string());
                                    v
                                }
                            })
                        },
                    }
                }
            },
            Rule::Max(max) => try!(match_max(max, self, field_title, raw)),
            Rule::Min(min) => try!(match_min(min, self, field_title, raw)),
            Rule::Format(format) => try!(match_format(format, self, field_title, raw)),
        }

    Ok(())

    }
}

fn match_max(max: i64, value: &FieldValue, field_title: &str, _: &str) -> Result<(), Message> {
    match *value {
        FieldValue::Str(ref s) => {
            if s.len() > max as usize {
                return Err(Message {
                    key: MessageKey::MaxLen,
                    values: {
                        let mut v = HashMap::new();
                        v.insert("name".to_string(), field_title.to_string());
                        v.insert("value".to_string(), max.to_string());
                        v
                    }
                })
            }
        },
        FieldValue::I64(i) => {
            if i > max {
                return Err(Message {
                    key: MessageKey::Max,
                    values: {
                        let mut v = HashMap::new();
                        v.insert("name".to_string(), field_title.to_string());
                        v.insert("value".to_string(), max.to_string());
                        v
                    }
                })
            }
        },
    }
    Ok(())
}

fn match_min(min: i64, value: &FieldValue, field_title: &str, _: &str) -> Result<(), Message> {
    match *value {
        FieldValue::Str(ref s) => {
            if s.len() < min as usize {
                return Err(Message {
                    key: MessageKey::MinLen,
                    values: {
                        let mut v = HashMap::new();
                        v.insert("name".to_string(), field_title.to_string());
                        v.insert("value".to_string(), min.to_string());
                        v
                    }
                })
            }
        },
        FieldValue::I64(i) => {
            if i < min {
                return Err(Message {
                    key: MessageKey::Min,
                    values: {
                        let mut v = HashMap::new();
                        v.insert("name".to_string(), field_title.to_string());
                        v.insert("value".to_string(), min.to_string());
                        v
                    }
                })
            }
        },
    }
    Ok(())
}

fn match_format(format: &str, value: &FieldValue, field_title: &str, _: &str) -> Result<(), Message> {
    let re = Regex::new(format).unwrap();
    if !re.is_match(&value.to_string()) {
        return Err(Message {
            key: MessageKey::Format,
            values: {
                let mut v = HashMap::new();
                v.insert("name".to_string(), field_title.to_string());
                v
            }
        })
    }
    Ok(())
}

pub struct Str;

impl FieldType for Str {
    fn from_str(&self, _: &str, value: &str) -> Result<FieldValue, Message> {
        Ok(FieldValue::Str(value.to_string()))
    }
}


pub struct I64;

impl FieldType for I64 {
    fn from_str(&self, field_title: &str, value: &str) -> Result<FieldValue, Message> {
        match value.to_string().parse::<i64>() {
            Ok(i) => Ok(FieldValue::I64(i)),
            Err(_) => Err(Message {
                key: MessageKey::Format,
                values: {
                    let mut v = HashMap::new();
                    v.insert("name".to_string(), field_title.to_string());
                    v
                }
            }),
        }
    }
}

pub struct ChinaMobile;

impl FieldType for ChinaMobile {
    fn from_str(&self, field_title: &str, value: &str) -> Result<FieldValue, Message> {
        let re = Regex::new(r"^1\d{10}$").unwrap();
        if !re.is_match(value) {
            return Err(Message {
                key: MessageKey::Format,
                values: {
                    let mut v = HashMap::new();
                    v.insert("name".to_string(), field_title.to_string());
                    v
                }
            })
        }
        Ok(FieldValue::Str(value.to_string()))
    }
}

pub struct Email;

impl FieldType for Email {
    fn from_str(&self, field_title: &str, value: &str) -> Result<FieldValue, Message> {
        let re = Regex::new(r"(?i)^[\w.%+-]+@(?:[A-Z0-9-]+\.)+[A-Z]{2,4}$").unwrap();
        if !re.is_match(value) {
            return Err(Message {
                key: MessageKey::Format,
                values: {
                    let mut v = HashMap::new();
                    v.insert("name".to_string(), field_title.to_string());
                    v
                }
            })
        }
        Ok(FieldValue::Str(value.to_string()))
    }
}
