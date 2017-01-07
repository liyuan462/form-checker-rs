extern crate regex;

use std::collections::HashMap;
use regex::Regex;

pub struct Validator<T: IntoMessage=()> {
    pub checkers: Vec<Checker>,
    pub valid_data: HashMap<String, Option<Vec<FieldValue>>>,
    pub invalid_messages: HashMap<String, String>,
    pub message: T,
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

    pub fn check(&mut self, checker: Checker) -> &mut Validator<T> {
        self.checkers.push(checker);
        self
    }

    pub fn validate(&mut self, params: &HashMap<String, Vec<String>>) {
        for checker in &self.checkers {
            match checker.check(params) {
                Ok(v) => {
                    self.valid_data.insert(checker.field_name.clone(), v);
                },
                Err(msg) => {
                    self.invalid_messages.insert(checker.field_name.clone(), msg.format_message(&self.message));
                },
            }
        }
    }

    pub fn get_required(&self, name: &str) -> FieldValue {
        (&(self.valid_data.get(name).unwrap().as_ref().unwrap()[0])).clone()
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

struct Message {
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

pub struct Checker {
    field_name: String,
    field_type: FieldType,
    rules: Vec<Rule>,
    optional: bool,
    multiple: bool,
}

impl Checker {
    pub fn new(field_name: &str, field_type: FieldType) -> Checker {
        Checker {
            field_name: field_name.to_string(),
            field_type: field_type,
            rules: Vec::new(),
            optional: false,
            multiple: false,
        }
    }

    fn check(&self, params: &HashMap<String, Vec<String>>) -> Result<Option<Vec<FieldValue>>, Message>{
        let values = params.get(&self.field_name);

        if values.is_none() {
            if !self.optional {
                return Err(Message {
                    key: MessageKey::Blank,
                    values: {
                        let mut v = HashMap::new();
                        v.insert("name".to_string(), self.field_name.clone());
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
                            v.insert("name".to_string(), self.field_name.clone());
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

    fn check_value(&self, value: &str) -> Result<FieldValue, Message> {
        let field_value = self.field_type.from_str(value);
        if field_value.is_none() {
            return Err(Message {
                key: MessageKey::Format,
                values: {
                    let mut v = HashMap::new();
                    v.insert("name".to_string(), self.field_name.clone());
                    v
                }
            })
        }
        let field_value = field_value.unwrap();
        for rule in &self.rules {
            if let Err(msg) = field_value.match_rule(self, rule) {
                return Err(msg);
            }
        }
        Ok(field_value)
    }

    pub fn meet(mut self, rule: Rule) -> Checker {
        self.rules.push(rule);
        self
    }

    pub fn set(mut self, option: CheckerOption) -> Checker {
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

pub enum FieldType {
    Str,
    Int,
}

#[derive(Clone)]
pub enum FieldValue {
    StrValue(String),
    IntValue(i64),
}

impl FieldType {
    fn from_str(&self, value: &str) -> Option<FieldValue> {
        match *self {
            FieldType::Str => {
                Some(FieldValue::StrValue(value.to_string()))
            },
            FieldType::Int => {
                match value.to_string().parse::<i64>() {
                    Ok(i) => Some(FieldValue::IntValue(i)),
                    Err(_) => None,
                }
            },

        }
    }
}

impl FieldValue {
    fn match_rule(&self, checker: &Checker, rule: &Rule) -> Result<(), Message> {
        match *self {
            FieldValue::StrValue(ref s) => {
                match *rule {
                    Rule::Max(max) => {
                        if s.len() > max as usize {
                            return Err(Message {
                                key: MessageKey::MaxLen,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), checker.field_name.clone());
                                    v.insert("value".to_string(), max.to_string());
                                    v
                                }
                            })
                        }
                    },
                    Rule::Min(min) => {
                        if s.len() < min as usize {
                            return Err(Message {
                                key: MessageKey::MinLen,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), checker.field_name.clone());
                                    v.insert("value".to_string(), min.to_string());
                                    v
                                }
                            })
                        }
                    },
                    Rule::Format(format) => {
                        let re = Regex::new(format).unwrap();
                        if !re.is_match(s) {
                            return Err(Message {
                                key: MessageKey::Format,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), checker.field_name.clone());
                                    v
                                }
                            })
                        }
                    },
                    Rule::Lambda(ref f, ref err_handler) => {
                        if !f(FieldValue::StrValue(s.clone())) {
                            match *err_handler {
                                Some(ref handler) => {
                                    return Err(Message {
                                        key: MessageKey::Custom,
                                        values: {
                                            let mut v = HashMap::new();
                                            v.insert("value".to_string(), handler(&checker.field_name, &s));
                                            v
                                        }
                                    })
                                },
                                None => {
                                    return Err(Message {
                                        key: MessageKey::Format,
                                        values: {
                                            let mut v = HashMap::new();
                                            v.insert("name".to_string(), checker.field_name.clone());
                                            v
                                        }
                                    })
                                },
                            }
                        }
                    }
                }
            }
            FieldValue::IntValue(i) => {
                match *rule {
                    Rule::Max(max) => {
                        if i > max {
                            return Err(Message {
                                key: MessageKey::Max,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), checker.field_name.clone());
                                    v.insert("value".to_string(), max.to_string());
                                    v
                                }
                            })
                        }
                    },
                    Rule::Min(min) => {
                        if i < min {
                            return Err(Message {
                                key: MessageKey::Min,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), checker.field_name.clone());
                                    v.insert("value".to_string(), min.to_string());
                                    v
                                }
                            })
                        }
                    },
                    Rule::Format(format) => {
                        let re = Regex::new(format).unwrap();
                        if !re.is_match(&i.to_string()) {
                            return Err(Message {
                                key: MessageKey::Format,
                                values: {
                                    let mut v = HashMap::new();
                                    v.insert("name".to_string(), checker.field_name.clone());
                                    v
                                }
                            })
                        }
                    },
                    Rule::Lambda(ref f, ref err_handler) => {
                        if !f(FieldValue::IntValue(i)) {
                            match *err_handler {
                                Some(ref handler) => {
                                    return Err(Message {
                                        key: MessageKey::Custom,
                                        values: {
                                            let mut v = HashMap::new();
                                            v.insert("value".to_string(), handler(&checker.field_name, &i.to_string()));
                                            v
                                        }
                                    })
                                },
                                None => {
                                    return Err(Message {
                                        key: MessageKey::Format,
                                        values: {
                                            let mut v = HashMap::new();
                                            v.insert("name".to_string(), checker.field_name.clone());
                                            v
                                        }
                                    })
                                },
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn as_str(&self) -> Option<String> {
        match *self {
            FieldValue::StrValue(ref s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match *self {
            FieldValue::IntValue(i) => Some(i),
            _ => None,
        }
    }

}
