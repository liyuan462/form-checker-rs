//! A library to check the values from a submitted form or a query string.
//!
//! This library provides a validator, with the hope of helping Web
//! developers check user-submitted values in an easy and declarative way.
//!
//! # Examples
//!
//! ```
//! use form_checker::{Validator, Checker, Rule, Str};
//! use std::collections::HashMap;
//!
//! let mut validator = Validator::new();
//! validator
//!     .check(Checker::new("username", "username", Str)
//!            .meet(Rule::Max(5))
//!            .meet(Rule::Min(2)));
//!
//! let mut params = std::collections::HashMap::new();
//! params.insert("username".to_string(), vec!["bob".to_string()]);
//! validator.validate(&params);
//! assert_eq!(validator.get_required("username").as_str().unwrap(), "bob".to_string());
//! ```

extern crate regex;

use std::collections::HashMap;
use regex::Regex;

pub struct Validator<T: IntoMessage=()> {
    pub checkers: Vec<Box<Checkable>>,
    pub valid_data: HashMap<String, Option<Vec<Box<FieldValue>>>>,
    pub invalid_messages: HashMap<String, String>,
    pub message: T,
}

pub trait Checkable {
    fn check(&self, params: &HashMap<String, Vec<String>>) -> Result<Option<Vec<Box<FieldValue>>>, Message>;

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

    pub fn get_required(&self, name: &str) -> Primitive {
        (&(self.valid_data.get(name).unwrap().as_ref().unwrap()[0])).to_primitive()
    }

    pub fn get_optional(&self, name: &str) -> Option<Primitive> {
        match self.valid_data.get(name).unwrap().as_ref() {
            Some(v) => Some(v[0].to_primitive()),
            None => None,
        }
    }

    pub fn get_required_multiple(&self, name: &str) -> Vec<Primitive> {
        self.valid_data.get(name).unwrap().as_ref().unwrap().iter().map(|item| item.to_primitive()).collect()
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
    fn check(&self, params: &HashMap<String, Vec<String>>) -> Result<Option<Vec<Box<FieldValue>>>, Message> {
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
                    Ok(v) => valid_values.push(Box::new(v) as Box<FieldValue>),
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
                Ok(v) => valid_values.push(Box::new(v) as Box<FieldValue>),
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

    fn check_value(&self, value: &str) -> Result<T::Value, Message> {
        let field_value = self.field_type.from_str(value);
        if field_value.is_none() {
            return Err(Message {
                key: MessageKey::Format,
                values: {
                    let mut v = HashMap::new();
                    v.insert("name".to_string(), self.field_title.clone());
                    v
                }
            })
        }
        let field_value = field_value.unwrap();
        for rule in &self.rules {
            if let Err(msg) = field_value.match_rule(&self.field_title, rule) {
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
    Lambda(Box<Fn(Primitive) -> bool>, Option<Box<Fn(&str, &str) -> String>>)
}

pub trait FieldType {
    type Value: FieldValue + Clone + 'static;

    fn from_str(&self, value: &str) -> Option<Self::Value>;
}

pub trait FieldValue {
    fn match_rule(&self, field_title: &str, rule: &Rule) -> Result<(), Message>;
    fn to_primitive(&self) -> Primitive;
}

pub enum Primitive {
    Str(String),
    I64(i64),
}

impl Primitive {
    pub fn as_str(&self) -> Option<String> {
        match *self {
            Primitive::Str(ref s) => Some(s.clone()),
            _ => None
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Primitive::I64(i) => Some(i),
            _ => None
        }
    }

}

pub struct Str;

#[derive(Clone)]
pub struct StrValue(String);

impl FieldType for Str {
    type Value = StrValue;

    fn from_str(&self, value: &str) -> Option<StrValue> {
        Some(StrValue(value.to_string()))
    }
}

impl FieldValue for StrValue {
    fn to_primitive(&self) -> Primitive {
        Primitive::Str(self.0.clone())
    }

    fn match_rule(&self, field_title: &str, rule: &Rule) -> Result<(), Message> {
        match *rule {
            Rule::Max(max) => {
                if self.0.len() > max as usize {
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
            Rule::Min(min) => {
                if self.0.len() < min as usize {
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
            Rule::Format(format) => {
                let re = Regex::new(format).unwrap();
                if !re.is_match(&self.0) {
                    return Err(Message {
                        key: MessageKey::Format,
                        values: {
                            let mut v = HashMap::new();
                            v.insert("name".to_string(), field_title.to_string());
                            v
                        }
                    })
                }
            },
            Rule::Lambda(ref f, ref err_handler) => {
                if !f(self.to_primitive()) {
                    match *err_handler {
                        Some(ref handler) => {
                            return Err(Message {
                                key: MessageKey::Custom,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("value".to_string(), handler(field_title, &self.0));
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
            }
        }

        Ok(())
    }
}

pub struct I64;

#[derive(Clone)]
pub struct I64Value(i64);

impl FieldType for I64 {
    type Value = I64Value;
    fn from_str(&self, value: &str) -> Option<I64Value> {
        match value.to_string().parse::<i64>() {
            Ok(i) => Some(I64Value(i)),
            Err(_) => None,
        }
    }
}

impl FieldValue for I64Value {
    fn to_primitive(&self) -> Primitive {
        Primitive::I64(self.0)
    }

    fn match_rule(&self, field_title: &str, rule: &Rule) -> Result<(), Message> {
        match *rule {
            Rule::Max(max) => {
                if self.0 > max {
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
            Rule::Min(min) => {
                if self.0 < min {
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
            Rule::Format(format) => {
                let re = Regex::new(format).unwrap();
                if !re.is_match(&self.0.to_string()) {
                    return Err(Message {
                        key: MessageKey::Format,
                        values: {
                            let mut v = HashMap::new();
                            v.insert("name".to_string(), field_title.to_string());
                            v
                        }
                    })
                }
            },
            Rule::Lambda(ref f, ref err_handler) => {
                if !f(self.to_primitive()) {
                    match *err_handler {
                        Some(ref handler) => {
                            return Err(Message {
                                key: MessageKey::Custom,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("value".to_string(), handler(field_title, &self.0.to_string()));
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
            }
        }

        Ok(())
    }
}

