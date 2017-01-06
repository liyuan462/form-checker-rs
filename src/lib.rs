extern crate regex;

use std::ops::Shl;
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

    pub fn add_checker(&mut self, checker: Checker) -> &mut Validator<T> {
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
    MaxLen,
    MinLen,
    Blank,
    Format,
}

pub trait IntoMessage {
    fn max_len(&self, name: &str, value: &str) -> String;
    fn min_len(&self, name: &str, value: &str) -> String;
    fn blank(&self, name: &str) -> String;
    fn format(&self, name: &str) -> String;
}

impl IntoMessage for () {
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

struct Message {
    key: MessageKey,
    values: HashMap<String, String>,
}

impl Message {
    fn format_message<T: IntoMessage>(&self, m: &T) -> String {
        match self.key {
            MessageKey::MaxLen => m.max_len(self.values.get("name").unwrap(), self.values.get("value").unwrap()),
            MessageKey::MinLen => m.min_len(self.values.get("name").unwrap(), self.values.get("value").unwrap()),
            MessageKey::Blank => m.blank(self.values.get("name").unwrap()),
            MessageKey::Format => m.format(self.values.get("name").unwrap()),
        }
    }
}

#[derive(Clone)]
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
}

impl Shl<Rule> for Checker {

    type Output = Checker;

    fn shl(self, rule: Rule) -> Checker {
        let mut checker = self.clone();

        match rule {
            Rule::Optional => checker.optional = true,
            Rule::Multiple => checker.multiple = true,
            _ => checker.rules.push(rule),
        }

        checker
    }
}

#[derive(Clone)]
pub enum Rule {
    Max(i64),
    Min(i64),
    Format(&'static str),
    Optional,
    Multiple,
}

#[derive(Clone)]
pub enum FieldType {
    Str,
}

#[derive(Clone)]
pub enum FieldValue {
    StrValue(String),
}

impl FieldType {
    fn from_str(&self, value: &str) -> Option<FieldValue> {
        match *self {
            FieldType::Str => {
                Some(FieldValue::StrValue(value.to_string()))
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
                    }
                    _ => {},
                }
            }
        }
        Ok(())
    }

    pub fn as_str(&self) -> Option<String> {
        match *self {
            FieldValue::StrValue(ref s) => Some(s.clone()),
         }
    }

}