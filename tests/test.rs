extern crate form_checker;

use std::collections::HashMap;
use form_checker::{Validator, Checker, FieldType, Rule, IntoMessage};

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

#[test]
fn other_message_lang() {
    struct MyMessage;
    impl IntoMessage for MyMessage {
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
