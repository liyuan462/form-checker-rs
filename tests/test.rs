extern crate form_checker;

use std::collections::HashMap;
use form_checker::{Validator, Checker, FieldType, Rule, IntoMessage, CheckerOption, FieldValue};

#[test]
fn check_str() {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("username", FieldType::Str)
                     << Rule::Max(5)
                     << Rule::Min(2));

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bob".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("username").as_str().unwrap(), "bob".to_string());

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();

    params.insert("username".to_string(), vec!["b".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "username长度不能小于2");

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["hellokitty".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "username长度不能大于5");

}

#[test]
fn check_str_format() {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("username", FieldType::Str)
                     << Rule::Format(r"l\dy"));

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["hellokitty".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "username格式不正确");

    validator.reset();

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["l5y".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 0);
    assert_eq!(validator.get_required("username").as_str().unwrap(), "l5y".to_string());
}

struct MyMessage;
impl IntoMessage for MyMessage {
    fn max(&self, name: &str, value: &str) -> String {
        format!("{name} can't be more than {value}", name=name, value=value)
    }

    fn min(&self, name: &str, value: &str) -> String {
        format!("{name} can't be less than {value}", name=name, value=value)
    }

    fn max_len(&self, name: &str, value: &str) -> String {
        format!("{name} can't be longer than {value}", name=name, value=value)
    }

    fn min_len(&self, name: &str, value: &str) -> String {
        format!("{name} can't be shorter than {value}", name=name, value=value)
    }

    fn blank(&self, name: &str) -> String {
        format!("{name} is missing", name=name)
    }
    fn format(&self, name: &str) -> String {
        format!("{name} is in wrong format", name=name)
    }
}

#[test]
fn other_message_lang() {

    let mut validator = Validator::with_message(MyMessage);
    validator
        .add_checker(Checker::new("username", FieldType::Str)
                     << Rule::Format(r"l\dy"));

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["hellokitty".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "username is in wrong format");
}

#[test]
fn check_optional() {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("username", FieldType::Str)
                     << CheckerOption::Optional(true)
                     << Rule::Max(5)
                     << Rule::Min(2));

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bcc".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_optional("username").unwrap().as_str().unwrap(), "bcc".to_string());

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    validator.validate(&params);
    assert!(validator.get_optional("username").is_none());

}


#[test]
fn check_int() {
    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("age", FieldType::Int)
                     << Rule::Max(5)
                     << Rule::Min(2));

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    params.insert("age".to_string(), vec!["3".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("age").as_int().unwrap(), 3);

    ////////////////////////////////////////////////

    validator.reset();

    let mut params = HashMap::new();

    params.insert("age".to_string(), vec!["1".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("age"), "age不能小于2");

    ////////////////////////////////////////////////

    validator.reset();

    let mut params = HashMap::new();
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("age"), "age不能为空");

    ////////////////////////////////////////////////

    validator.reset();

    let mut params = HashMap::new();
    params.insert("age".to_string(), vec!["".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("age"), "age格式不正确");

    let mut validator = Validator::new();
    validator
        .add_checker(Checker::new("age", FieldType::Int)
                     << Rule::Format(r"\d{4}"));

    let mut params = HashMap::new();
    params.insert("age".to_string(), vec!["3456".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("age").as_int().unwrap(), 3456);

    ////////////////////////////////////////////////

    validator.reset();
    let mut params = HashMap::new();
    params.insert("age".to_string(), vec!["345".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("age"), "age格式不正确");

}

#[test]
fn check_lambda() {
    struct Lambda(Box<Fn(FieldValue) -> bool>);
    let l = Lambda(Box::new(|v| true));
}
