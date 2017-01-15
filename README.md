# form-checker-rs

A library for Web developers to check the values from a submitted form or a query string. 

## Install

```toml
# Cargo.toml
[dependencies]
form-checker = "0.2"
```

## Example

```rust
extern crate form_checker;

use form_checker::{Validator, Checker, Rule, Str, I64};

fn main() {
    // Prepare params, this is just for illustrating. Usually, we get
    // params through decoding a URL-encoded string into a
    // HashMap<String, Vec<String>>.
    let mut params = std::collections::HashMap::new();
    params.insert("name".to_string(), vec!["bob".to_string()]);
    params.insert("age".to_string(), vec!["20".to_string()]);
   
    // Make a new Validator.
    let mut validator = Validator::new();
    // Add Checkers to Validator.
    validator
        .check(Checker::new("name", "姓名", Str)
               .meet(Rule::Max(5))
               .meet(Rule::Min(2)))
        .check(Checker::new("age", "年龄", I64)
               .meet(Rule::Max(100))
               .meet(Rule::Min(18)));
    // Validate it!
    validator.validate(&params);
    // Decide whether it is valid.
    assert!(validator.is_valid());
    // Show me the valid data, assuming it is valid.
    assert_eq!(validator.get_required("name").as_str().unwrap(), "bob".to_string());
    assert_eq!(validator.get_required("age").as_i64().unwrap(), 20);
}
```

## [Full Documentation](https://docs.rs/form-checker/0.2.2/form_checker/)

## License

`form-checker-rs` is primarily distributed under the terms of the MIT license.

See LICENSE-MIT for details.
