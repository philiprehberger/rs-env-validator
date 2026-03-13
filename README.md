# rs-env-validator

Typed environment variable validation with batch error reporting for Rust.

## Installation

```toml
[dependencies]
philiprehberger-env-validator = "0.3"
```

## Usage

### Basic Validation

```rust
use philiprehberger_env_validator::Schema;

let config = Schema::new()
    .string("DATABASE_URL").build()
    .integer("PORT").default_value("3000").build()
    .boolean("DEBUG").default_value("false").build()
    .validate()?;

let port = config["PORT"].as_int().unwrap();
```

### Float and URL Fields

```rust
let config = Schema::new()
    .float("RATE_LIMIT").default_value("1.5").build()
    .url("API_ENDPOINT").build()
    .validate()?;

let rate = config["RATE_LIMIT"].as_float().unwrap();
let endpoint = config["API_ENDPOINT"].as_str().unwrap();
```

### With Choices

```rust
let config = Schema::new()
    .string("APP_ENV").choices(&["development", "staging", "production"]).build()
    .validate()?;
```

### Custom Source (Testing)

```rust
use std::collections::HashMap;

let mut source = HashMap::new();
source.insert("PORT".to_string(), "8080".to_string());

let config = Schema::new()
    .integer("PORT").build()
    .validate_from(Some(&source))?;
```

### Error Handling

```rust
match schema.validate() {
    Ok(config) => { /* use config */ }
    Err(e) => {
        for error in &e.errors {
            eprintln!("{}", error);
        }
    }
}
```

### Type Conversions

```rust
use philiprehberger_env_validator::EnvValue;

let val: EnvValue = "hello".into();
let val: EnvValue = 42i64.into();
let val: EnvValue = 3.14f64.into();
let val: EnvValue = true.into();

// Display
println!("{}", val); // "true"

// Compare
assert_eq!(EnvValue::from(42i64), EnvValue::Int(42));
```

## License

MIT
