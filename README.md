# rs-env-validator

[![CI](https://github.com/philiprehberger/rs-env-validator/actions/workflows/ci.yml/badge.svg)](https://github.com/philiprehberger/rs-env-validator/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/philiprehberger-env-validator.svg)](https://crates.io/crates/philiprehberger-env-validator)
[![Last updated](https://img.shields.io/github/last-commit/philiprehberger/rs-env-validator)](https://github.com/philiprehberger/rs-env-validator/commits/main)

Typed environment variable validation with batch error reporting for Rust

## Installation

```toml
[dependencies]
philiprehberger-env-validator = "0.4.0"
```

## Usage

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

### Numeric Range

```rust
let config = Schema::new()
    .integer("PORT").min(1024.0).max(65535.0).build()
    .float("RATE").min(0.0).max(1.0).build()
    .validate()?;
```

### String Length

```rust
let config = Schema::new()
    .string("PASSWORD").min_length(8).max_length(64).build()
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

## API

| Function / Type | Description |
|-----------------|-------------|
| `Schema::new()` | Create a new empty validation schema |
| `schema.string(name)` | Add a string field to the schema |
| `schema.integer(name)` | Add an integer field to the schema |
| `schema.float(name)` | Add a float field to the schema |
| `schema.boolean(name)` | Add a boolean field to the schema |
| `schema.url(name)` | Add a URL field to the schema |
| `builder.required(bool)` | Set whether the field is required (default: true) |
| `builder.default_value(v)` | Set a default value for the field |
| `builder.choices(list)` | Restrict allowed values to a set of choices |
| `builder.min(v)` / `builder.max(v)` | Numeric range bounds (`integer` / `float`) |
| `builder.min_length(n)` / `builder.max_length(n)` | String length bounds (`string` / `url`) |
| `builder.build()` | Finalize the field and return the schema |
| `schema.validate()` | Validate from environment variables |
| `schema.validate_from(source)` | Validate from a custom `HashMap` source |
| `schema.field_count()` | Number of fields registered in the schema |
| `EnvValue` | Enum: `Str`, `Int`, `Float`, `Bool` |
| `EnvValue.as_string()` | Render as `String` regardless of variant |
| `ValidationError` | Error containing a `Vec<String>` of all failures |

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Support

If you find this project useful:

⭐ [Star the repo](https://github.com/philiprehberger/rs-env-validator)

🐛 [Report issues](https://github.com/philiprehberger/rs-env-validator/issues?q=is%3Aissue+is%3Aopen+label%3Abug)

💡 [Suggest features](https://github.com/philiprehberger/rs-env-validator/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement)

❤️ [Sponsor development](https://github.com/sponsors/philiprehberger)

🌐 [All Open Source Projects](https://philiprehberger.com/open-source-packages)

💻 [GitHub Profile](https://github.com/philiprehberger)

🔗 [LinkedIn Profile](https://www.linkedin.com/in/philiprehberger)

## License

[MIT](LICENSE)
