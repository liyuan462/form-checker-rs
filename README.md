# form-checker-rs

A library for Web developers to check the values from a submitted form or a query string. 

## example

```rust
extern crate form_checker;

use std::collections::HashMap;
use form_checker::{Validator, Checker, Rule, Str};

fn main() {
    let mut params = HashMap::new();
    params.insert("username".to_string(), vec!["bob".to_string()]);

    let mut validator = Validator::new();
    validator
        .check(Checker::new("username", "username", Str)
              .meet(Rule::Max(5))
              .meet(Rule::Min(2)));

    validator.validate(&params);
    
    assert_eq!(validator.get_required("username").as_str().unwrap(), "bob".to_string());
}
```
