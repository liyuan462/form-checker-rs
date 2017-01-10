extern crate form_checker;

use std::collections::HashMap;
use form_checker::{Validator, Checker, Rule, IntoMessage, CheckerOption, Str, I64, ChinaMobile, Email};

#[test]
fn check_str() {
    let mut validator = Validator::new();
    validator
        .check(Checker::new("username", "username", Str)
              .meet(Rule::Max(5))
              .meet(Rule::Min(2)));

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
        .check(Checker::new("username", "username", Str)
                     .meet(Rule::Format(r"l\dy")));

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
        .check(Checker::new("username", "username", Str)
                     .meet(Rule::Format(r"l\dy")));

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
        .check(Checker::new("username", "username", Str)
                     .set(CheckerOption::Optional(true))
                     .meet(Rule::Max(5))
                     .meet(Rule::Min(2)));

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bcc".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_optional("username").unwrap().as_str().unwrap(), "bcc".to_string());

    ////////////////////////////////////////////////
    validator.reset();

    let params = HashMap::new();
    validator.validate(&params);
    assert!(validator.get_optional("username").is_none());

}

#[test]
fn check_i64() {
    let mut validator = Validator::new();
    validator
        .check(Checker::new("age", "age", I64)
                     .meet(Rule::Max(5))
                     .meet(Rule::Min(2)));

    ////////////////////////////////////////////////
    validator.reset();

    let mut params = HashMap::new();
    params.insert("age".to_string(), vec!["3".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("age").as_i64().unwrap(), 3);

    ////////////////////////////////////////////////

    validator.reset();

    let mut params = HashMap::new();

    params.insert("age".to_string(), vec!["1".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("age"), "age不能小于2");

    ////////////////////////////////////////////////

    validator.reset();

    let params = HashMap::new();
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
        .check(Checker::new("age", "age", I64)
                     .meet(Rule::Format(r"\d{4}")));

    let mut params = HashMap::new();
    params.insert("age".to_string(), vec!["3456".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("age").as_i64().unwrap(), 3456);

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
    let mut validator = Validator::new();
    validator.check(Checker::new("username", "username", Str)
                    .meet(Rule::Lambda(Box::new(|_| true), None)));

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bob".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("username").as_str().unwrap(), "bob".to_string());

    let mut validator = Validator::new();
    validator.check(Checker::new("username", "username", Str)
                    .meet(Rule::Lambda(Box::new(|v| v.as_str().unwrap().len() == 3), None)));

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bob".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.get_required("username").as_str().unwrap(), "bob".to_string());

    validator.reset();
    let mut params = HashMap::new();

    params.insert("username".to_string(), vec!["b".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "username格式不正确");

    let mut validator = Validator::new();
    validator.check(Checker::new("username", "username", Str)
                    .meet(Rule::Lambda(Box::new(|v| v.as_str().unwrap().len() == 3),
                    Some(Box::new(|name, value| format!("{}格式不对:{}", name, value))))));

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["b".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "username格式不对:b");

}

#[test]
fn check_title() {
    let mut validator = Validator::new();
    validator.check(Checker::new("username", "用户名", Str)
                    .meet(Rule::Max(5))
                    .meet(Rule::Min(2)));

    let mut params = HashMap::new();

    params.insert("username".to_string(), vec!["b".to_string()]);
    validator.validate(&params);
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("username"), "用户名长度不能小于2");

}

#[test]
fn check_multiple() {
    let mut validator = Validator::new();
    validator.check(Checker::new("username", "用户名", Str)
                    .set(CheckerOption::Multiple(true))
                    .meet(Rule::Max(5))
                    .meet(Rule::Min(2)));

    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bob".to_string(), "mary".to_string()]);
    validator.validate(&params);
    assert!(validator.is_valid());
    assert_eq!(validator.get_required_multiple("username").iter().map(|item| item.as_str().unwrap()).collect::<Vec<_>>(), vec!["bob".to_string(), "mary".to_string()]);

    ////////////////////////////////////////////////

    validator.reset();
    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bob".to_string(), "i".to_string()]);
    validator.validate(&params);
    assert!(!validator.is_valid());
    assert_eq!(validator.get_error("username"), "用户名长度不能小于2");
}

#[test]
fn check_china_mobile() {
    let mut validator = Validator::new();
    validator.check(Checker::new("mobile", "手机", ChinaMobile));

    let mut params = HashMap::new();
    params.insert("mobile".to_string(), vec!["13334567890".to_string()]);
    validator.validate(&params);
    assert!(validator.is_valid());

    ////////////////////////////////////////////////

    validator.reset();
    let mut params = HashMap::new();
    params.insert("mobile".to_string(), vec!["23444".to_string()]);
    validator.validate(&params);
    assert!(!validator.is_valid());
    assert_eq!(validator.get_error("mobile"), "手机格式不正确");

}

#[test]
fn check_email() {
    let mut validator = Validator::new();
    validator.check(Checker::new("email", "邮箱", Email));

    let mut params = HashMap::new();
    params.insert("email".to_string(), vec!["abb@howadata.com".to_string()]);
    validator.validate(&params);
    assert!(validator.is_valid());

    ////////////////////////////////////////////////

    validator.reset();
    let mut params = HashMap::new();
    params.insert("email".to_string(), vec!["abb@howadata".to_string()]);
    validator.validate(&params);
    assert!(!validator.is_valid());
    assert_eq!(validator.get_error("email"), "邮箱格式不正确");

}

#[test]
fn multi_checkers() {
    let mut validator = Validator::new();
    validator.check(Checker::new("email", "邮箱", Email))
        .check(Checker::new("mobile", "手机", ChinaMobile));

    let mut params = HashMap::new();
    params.insert("email".to_string(), vec!["abb@howadata.com".to_string()]);
    validator.validate(&params);
    assert!(!validator.is_valid());
    assert_eq!(validator.invalid_messages.len(), 1);
    assert_eq!(validator.get_error("mobile"), "手机不能为空");

    ////////////////////////////////////////////////

    validator.reset();
    params.insert("email".to_string(), vec!["abb@howadata.com".to_string()]);
    params.insert("mobile".to_string(), vec!["13334567890".to_string()]);
    validator.validate(&params);
    assert!(validator.is_valid());
}
