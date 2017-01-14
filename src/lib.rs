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

/// The Validator type.
///
/// Each time we want to validate form values, we make a validator.
///
/// Firstly, we add checkers to it calling its `check` method.
///
/// Then, we call its `validator` method to do the validating thing.
///
/// Finally, we get valid keys and values from its `valid_data` member and get invalid
/// keys and messages from  its `invalid_messages` member.
///
/// The `message_renderer` member is used to custom invalid messages.
pub struct Validator<T: MessageRenderer=()> {
    pub checkers: Vec<Box<Checkable>>,
    pub valid_data: HashMap<String, Option<Vec<FieldValue>>>,
    pub invalid_messages: HashMap<String, String>,
    pub message_renderer: T,
}

/// Represents a type to fed to the Validator.
pub trait Checkable {
    fn check(&self, params: &HashMap<String, Vec<String>>) -> Result<Option<Vec<FieldValue>>, Message>;
    fn get_name(&self) -> String;
}

impl Validator<()> {
    /// Constructs a new `Validator` with the default message renderer.
    pub fn new() -> Validator<()> {
        Validator::with_message(())
    }
}

impl<T: MessageRenderer> Validator<T> {
    /// Constructs a new `Validator` with a custom message renderer.
    ///
    /// Often, when there are invalid values, we want to show the users why.
    /// And you might want to custom the format for kinds of messages, especially,
    /// in your native language.
    /// You do it by giving a `MessageRenderer`.
    ///
    /// The default message renderer uses Simple Chinese.
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, MessageRenderer, MessageKind, SomeMessage};
    /// struct EnglishMessageRenderer;
    /// impl MessageRenderer for EnglishMessageRenderer {
    ///     fn render_message(&self, m: SomeMessage) -> String {
    ///         match m.kind {
    ///             MessageKind::Max => format!("{title} can't be more than {rule}", title=m.title, rule=m.rule_values[0]),
    ///             MessageKind::Min => format!("{title} can't be less than {rule}", title=m.title, rule=m.rule_values[0]),
    ///             MessageKind::MaxLen => format!("{title} can't be longer than {rule}", title=m.title, rule=m.rule_values[0]),
    ///             MessageKind::MinLen => format!("{title} can't be shorter than {rule}", title=m.title, rule=m.rule_values[0]),
    ///             MessageKind::Blank => format!("{title} is missing", title=m.title),
    ///             MessageKind::Format => format!("{title} is in wrong format", title=m.title),
    ///         }
    ///     }
    /// }
    /// let mut validator = Validator::with_message(EnglishMessageRenderer);
    /// ```
    pub fn with_message(message_renderer: T) -> Validator<T> {
        Validator {
            checkers: Vec::new(),
            valid_data: HashMap::new(),
            invalid_messages: HashMap::new(),
            message_renderer: message_renderer,
        }
    }

    /// Add a checker to this validator.
    ///
    /// We can chain this call to add multiple checkers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, Checker, Rule, Str, I64};
    /// let mut validator = Validator::new();
    /// // Add Checkers to Validator.
    /// validator
    ///     .check(Checker::new("name", "姓名", Str)
    ///            .meet(Rule::Max(5))
    ///            .meet(Rule::Min(2)))
    ///     .check(Checker::new("age", "年龄", I64)
    ///            .meet(Rule::Max(100))
    ///            .meet(Rule::Min(18)));
    /// ```
    pub fn check<U: Checkable + 'static>(&mut self, checker: U) -> &mut Validator<T> {
        self.checkers.push(Box::new(checker));
        self
    }

    /// Do the validating logic.
    ///
    /// Don't forget to add checkers first.
    pub fn validate(&mut self, params: &HashMap<String, Vec<String>>) {
        for checker in &self.checkers {
            match checker.check(params) {
                Ok(v) => {
                    self.valid_data.insert(checker.get_name().clone(), v);
                },
                Err(msg) => {
                    self.invalid_messages.insert(checker.get_name().clone(),
                                                 self.message_renderer.render(msg));
                },
            }
        }
    }

    /// Get a required valid value after validating.
    ///
    /// # Panics
    ///
    /// Call this method when you're sure the value exists and is valid, or it panics!
    ///
    /// You may want to call `is_valid` method first, when that is true,
    /// you have confidence that this call will not panic!
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, Checker, Rule, Str};
    /// let mut params = std::collections::HashMap::new();
    /// params.insert("name".to_string(), vec!["bob".to_string()]);
    ///
    /// let mut validator = Validator::new();
    /// validator
    ///     .check(Checker::new("name", "姓名", Str)
    ///            .meet(Rule::Max(5))
    ///            .meet(Rule::Min(2)));
    /// validator.validate(&params);
    /// assert!(validator.is_valid());
    /// assert_eq!(validator.get_required("name").as_str().unwrap(), "bob".to_string());
    /// ```
    pub fn get_required(&self, name: &str) -> FieldValue {
        self.valid_data.get(name).unwrap().as_ref().unwrap()[0].clone()
    }

    /// Get a optional valid value after validating.
    ///
    /// This method is for getting **optional** values, the value which is allowed
    /// to be missing.
    ///
    /// Refer to `CheckerOption::Optional`
    ///
    /// # Panics
    ///
    /// You may want to call `is_valid` method first, when that is true,
    /// you have confidence that this call will not panic!
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, Checker, CheckerOption, Rule, Str};
    /// let mut params = std::collections::HashMap::new();
    ///
    /// let mut validator = Validator::new();
    /// validator
    ///     .check(Checker::new("name", "姓名", Str)
    ///            .set(CheckerOption::Optional(true))
    ///            .meet(Rule::Max(5))
    ///            .meet(Rule::Min(2)));
    /// validator.validate(&params);
    /// assert!(validator.is_valid());
    /// assert!(validator.get_optional("name").is_none());
    /// ```
    pub fn get_optional(&self, name: &str) -> Option<FieldValue> {
        match self.valid_data.get(name).unwrap().as_ref() {
            Some(v) => Some(v[0].clone()),
            None => None,
        }
    }

    /// Get multiple valid values after validating.
    ///
    /// Sometimes we need a vector of values which have the same field name.
    ///
    /// This method is for getting **required** **multiple** values
    ///
    /// Refer to `CheckerOption::Multiple`
    ///
    /// # Panics
    ///
    /// You may want to call `is_valid` method first, when that is true,
    /// you have confidence that this call will not panic!
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, Checker, CheckerOption, Rule, Str};
    /// let mut params = std::collections::HashMap::new();
    /// params.insert("name".to_string(), vec!["bob".to_string(), "mary".to_string()]);
    ///
    /// let mut validator = Validator::new();
    /// validator
    ///     .check(Checker::new("name", "姓名", Str)
    ///            .set(CheckerOption::Multiple(true))
    ///            .meet(Rule::Max(5))
    ///            .meet(Rule::Min(2)));
    /// validator.validate(&params);
    /// assert!(validator.is_valid());
    /// assert_eq!(validator.get_required_multiple("name").iter().map(|item| item.as_str().unwrap()).collect::<Vec<_>>(), vec!["bob".to_string(), "mary".to_string()]);
    /// ```
    pub fn get_required_multiple(&self, name: &str) -> Vec<FieldValue> {
        self.valid_data.get(name).unwrap().as_ref().unwrap().clone()
     }

    /// Get optional multiple valid values after validating.
    ///
    /// Sometimes we need a vector of values which have the same field name.
    ///
    /// This method is for getting **optional** **multiple** values
    ///
    /// Refer to `CheckerOption`
    ///
    /// # Panics
    ///
    /// You may want to call `is_valid` method first, when that is true,
    /// you have confidence that this call will not panic!
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, Checker, CheckerOption, Rule, Str};
    /// let mut params = std::collections::HashMap::new();
    /// params.insert("name".to_string(), vec!["bob".to_string(), "mary".to_string()]);
    ///
    /// let mut validator = Validator::new();
    /// validator
    ///     .check(Checker::new("name", "姓名", Str)
    ///            .set(CheckerOption::Multiple(true))
    ///            .set(CheckerOption::Optional(true))
    ///            .meet(Rule::Max(5))
    ///            .meet(Rule::Min(2)));
    /// validator.validate(&params);
    /// assert!(validator.is_valid());
    /// assert_eq!(validator.get_optional_multiple("name").unwrap().iter().map(|item| item.as_str().unwrap()).collect::<Vec<_>>(), vec!["bob".to_string(), "mary".to_string()]);
    /// ```
    pub fn get_optional_multiple(&self, name: &str) -> Option<Vec<FieldValue>> {
        self.valid_data.get(name).unwrap().clone()
    }

    /// Tell you whether the validator is valid or not, you must first call
    /// `validate` method.
    pub fn is_valid(&self) -> bool {
        self.valid_data.len() == self.checkers.len()
    }

    /// Get an error message.
    ///
    /// # Panics
    ///
    /// Make sure you know this field is invalid before you get its message,
    /// or it panics.
    ///
    /// # Examples
    ///
    /// ```
    /// # use form_checker::{Validator, Checker, Rule, Str};
    /// let mut params = std::collections::HashMap::new();
    /// params.insert("name".to_string(), vec!["b".to_string()]);
    ///
    /// let mut validator = Validator::new();
    /// validator
    ///     .check(Checker::new("name", "姓名", Str)
    ///            .meet(Rule::Max(5))
    ///            .meet(Rule::Min(2)));
    /// validator.validate(&params);
    /// assert!(!validator.is_valid());
    /// assert_eq!(validator.get_error("name"), "姓名长度不能小于2");
    /// ```
    pub fn get_error(&self, name: &str) -> String {
        self.invalid_messages.get(name).unwrap().clone()
    }

    /// Clear the valid_data and invalid_messages, as if you have not called `validate`.
    pub fn reset(&mut self) {
        self.valid_data.clear();
        self.invalid_messages.clear();
    }
}

/// This enum is used to mark a type of a `Message`
pub enum MessageKind {
    /// Greater than maximum value, eg. for an int value.
    Max,
    /// Less than mininum value, eg. for an int value.
    Min,
    /// Longer than maximum lenghth, eg. for a string value.
    MaxLen,
    /// Shorter than minimum length, eg. for a string value.
    MinLen,
    /// Value required, but missing.
    Blank,
    /// Value not match some format.
    Format,
}

/// A general message wrapper
pub enum Message {
    /// A customized message, just show any message given.
    Any(String),
    /// A kind of message predefined.
    Some(SomeMessage),
}

/// A specific message
pub struct SomeMessage {
    /// Refer to `MessageKind`.
    pub kind: MessageKind,
    /// The field name.
    pub name: String,
    /// The field title.
    pub title: String,
    /// The field raw value, None if missing.
    pub value: Option<String>,
    /// rule related values, such as max and min, as strings.
    pub rule_values: Vec<String>,
}

impl Message {
    /// Construct a kind of message.
    pub fn some(kind: MessageKind, name: &str, title: &str, value: Option<String>, rule_values: Vec<String>) -> Message {
        Message::Some(SomeMessage {
            kind: kind,
            name: name.to_string(),
            title: title.to_string(),
            value: value,
            rule_values: rule_values,
        })
    }

    /// Construct a customized message.
    pub fn any(message: &str) -> Message {
        Message::Any(message.to_string())
    }
}

/// If you want to control how the message is displayed, implement this trait.
///
/// The default implementation is in simple Chinese.
pub trait MessageRenderer {
    fn render_message(&self, m: SomeMessage) -> String {
        match m.kind {
            MessageKind::Max => format!("{title}不能大于{rule}", title=m.title, rule=m.rule_values[0]),
            MessageKind::Min => format!("{title}不能小于{rule}", title=m.title, rule=m.rule_values[0]),
            MessageKind::MaxLen => format!("{title}长度不能大于{rule}", title=m.title, rule=m.rule_values[0]),
            MessageKind::MinLen => format!("{title}长度不能小于{rule}", title=m.title, rule=m.rule_values[0]),
            MessageKind::Blank => format!("{title}不能为空", title=m.title),
            MessageKind::Format => format!("{title}格式不正确", title=m.title),
        }
    }
}

trait Renderable {
    fn render(&self, m:Message) -> String;
}

impl<T:MessageRenderer> Renderable for T {
    fn render(&self, m: Message) -> String {
        match m {
            Message::Any(s) => s,
            Message::Some(km) => self.render_message(km),
        }
    }
}

impl MessageRenderer for () {
}

/// Option you can set to a checker.
///
/// # Examples
///
/// ```
/// # use form_checker::{Validator, Checker, CheckerOption, Rule, Str};
/// let mut params = std::collections::HashMap::new();
/// params.insert("tags".to_string(), vec!["red".to_string(), "blue".to_string()]);
///
/// let mut validator = Validator::new();
/// validator
///     .check(Checker::new("tags", "标签", Str)
///            .set(CheckerOption::Optional(true))
///            .set(CheckerOption::Multiple(true))
///            .meet(Rule::Min(1)));
/// validator.validate(&params);
/// assert!(validator.is_valid());
/// ```
pub enum CheckerOption {
    /// True means this field is allowed to be missing, default false(required).
    Optional(bool),
    /// True means this field consists of multiple values, default false(single value).
    Multiple(bool),
}

/// The checker for a field.
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
                return Err(Message::some(MessageKind::Blank,
                                        &self.field_name,
                                        &self.field_title,
                                        None, Vec::new()));
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
                    return Err(Message::some(MessageKind::Blank,
                                            &self.field_name,
                                            &self.field_title,
                                            None, Vec::new()));
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
    /// Construct a new `Checker`.
    ///
    /// field_name is the field name in the form.
    ///
    /// field_title is a descriptive value, used to diplay error messages.
    ///
    /// field_type is a type implementing the `FieldType` trait.
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
        let field_value = try!(self.field_type.from_str(&self.field_name, &self.field_title, value));
        for rule in &self.rules {
            if let Err(msg) = field_value.match_rule(&self.field_name, &self.field_title, value, rule) {
                return Err(msg);
            }
        }
        Ok(field_value)
    }

    /// Add a rule to this checker, refer to the `Rule`.
    pub fn meet(mut self, rule: Rule) -> Checker<T> {
        self.rules.push(rule);
        self
    }

    /// Set an option for this checker, refer to the `CheckerOption`.
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

/// This enum offers rules avalable to be applied to a `FieldValue`.
///
/// Note that for diffent `FieldValue`, the same rule might mean diffent.
/// For example, Max means the maximum length for str value, but means
/// the maximum value for integer value.
pub enum Rule {
    /// Maximum limit.
    Max(i64),
    /// Mininum limit.
    Min(i64),
    /// A regex pattern to match against the str representation of `FieldValue`.
    Format(&'static str),
    /// A customized lambda, to let you offer your own check logic.
    Lambda(Box<Fn(FieldValue) -> bool>, Option<Box<Fn(&str, &str, &str) -> String>>)
}

/// This trait represents the field type.
///
/// We offer some field types, like `Str`, `I64`, `ChinaMobile` and `Email`.
///
/// You just need to implement this trait to transform the raw str value into a 
/// `FeildValue`.
///
/// field_name is the field name in the form.
///
/// field_title is a descriptive value, used to diplay invalid messages.
///
/// value is the raw str value from the form.
///
/// And of course you can implement your own field type!
pub trait FieldType {
    fn from_str(&self, field_name: &str, field_title: &str, value: &str) -> Result<FieldValue, Message>;
}

/// An enum to represent the primitive value extracted, resulting from applying
/// a checker.
#[derive(Clone)]
pub enum FieldValue {
    /// A str value.
    Str(String),
    /// An integer value as i64.
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
    /// Extract a str primitive from the `FieldValue`.
    pub fn as_str(&self) -> Option<String> {
        match *self {
            FieldValue::Str(ref s) => Some(s.clone()),
            _ => None
        }
    }

    /// Extract an i64 primitive from the `FieldValue`
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            FieldValue::I64(i) => Some(i),
            _ => None
        }
    }

    fn match_rule(&self, field_name: &str, field_title: &str, value: &str, rule: &Rule) -> Result<(), Message> {
        match *rule {
            Rule::Lambda(ref f, ref err_handler) => {
                if !f(self.clone()) {
                    match *err_handler {
                        Some(ref handler) => {
                            return Err(Message::any(&handler(field_name,
                                                            field_title,
                                                            value)));
                        },
                        None => {
                            return Err(Message::some(MessageKind::Format,
                                                    field_name,
                                                    field_title,
                                                    Some(value.to_string()),
                                                    Vec::new()));
                        },
                    }
                }
            },
            Rule::Max(max) => try!(match_max(max, self, field_name, field_title, value)),
            Rule::Min(min) => try!(match_min(min, self, field_name, field_title, value)),
            Rule::Format(format) => try!(match_format(format, self, field_name, field_title, value)),
        }

    Ok(())

    }
}

fn match_max(max: i64, value: &FieldValue, field_name: &str, field_title: &str, raw: &str) -> Result<(), Message> {
    match *value {
        FieldValue::Str(ref s) => {
            if s.len() > max as usize {
                return Err(Message::some(MessageKind::MaxLen,
                                        field_name,
                                        field_title,
                                        Some(raw.to_string()),
                                        vec![max.to_string()]));
            }
        },
        FieldValue::I64(i) => {
            if i > max {
                return Err(Message::some(MessageKind::Max,
                                        field_name,
                                        field_title,
                                        Some(raw.to_string()),
                                        vec![max.to_string()]));
            }
        },
    }
    Ok(())
}

fn match_min(min: i64, value: &FieldValue, field_name: &str, field_title: &str, raw: &str) -> Result<(), Message> {
    match *value {
        FieldValue::Str(ref s) => {
            if s.len() < min as usize {
                return Err(Message::some(MessageKind::MinLen,
                                        field_name,
                                        field_title,
                                        Some(raw.to_string()),
                                        vec![min.to_string()]));
            }
        },
        FieldValue::I64(i) => {
            if i < min {
                return Err(Message::some(MessageKind::Min,
                                        field_name,
                                        field_title,
                                        Some(raw.to_string()),
                                        vec![min.to_string()]));
            }
        },
    }
    Ok(())
}

fn match_format(format: &str, value: &FieldValue, field_name: &str, field_title: &str, raw: &str) -> Result<(), Message> {
    let re = Regex::new(format).unwrap();
    if !re.is_match(&value.to_string()) {
        return Err(Message::some(MessageKind::Format,
                                field_name,
                                field_title,
                                Some(raw.to_string()),
                                Vec::new()));
    }
    Ok(())
}

/// A general field type to represent a string field.
pub struct Str;

impl FieldType for Str {
    fn from_str(&self, _: &str, _: &str, value: &str) -> Result<FieldValue, Message> {
        Ok(FieldValue::Str(value.to_string()))
    }
}

/// A general field type to represent an integer field.
pub struct I64;

impl FieldType for I64 {
    fn from_str(&self, field_name: &str, field_title: &str, value: &str) -> Result<FieldValue, Message> {
        match value.to_string().parse::<i64>() {
            Ok(i) => Ok(FieldValue::I64(i)),
            Err(_) => Err(Message::some(MessageKind::Format,
                                    field_name,
                                    field_title,
                                    Some(value.to_string()),
                                    Vec::new())),
        }
    }
}

/// A field type to represent a mobile number used in China.
pub struct ChinaMobile;

impl FieldType for ChinaMobile {
    fn from_str(&self, field_name: &str, field_title: &str, value: &str) -> Result<FieldValue, Message> {
        let re = Regex::new(r"^1\d{10}$").unwrap();
        if !re.is_match(value) {
            return Err(Message::some(MessageKind::Format,
                                    field_name,
                                    field_title,
                                    Some(value.to_string()),
                                    Vec::new()));
        }
        Ok(FieldValue::Str(value.to_string()))
    }
}

/// A field type to represent an Email.
pub struct Email;

impl FieldType for Email {
    fn from_str(&self, field_name: &str, field_title: &str, value: &str) -> Result<FieldValue, Message> {
        let re = Regex::new(r"(?i)^[\w.%+-]+@(?:[A-Z0-9-]+\.)+[A-Z]{2,4}$").unwrap();
        if !re.is_match(value) {
            return Err(Message::some(MessageKind::Format,
                                    field_name,
                                    field_title,
                                    Some(value.to_string()),
                                    Vec::new()));
        }
        Ok(FieldValue::Str(value.to_string()))
    }
}
