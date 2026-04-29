//! Typed environment variable validation with batch error reporting.
//!
//! # Example
//!
//! ```rust,no_run
//! use philiprehberger_env_validator::Schema;
//!
//! let config = Schema::new()
//!     .string("DATABASE_URL").required(true).build()
//!     .string("LOG_LEVEL").default_value("info").build()
//!     .validate()
//!     .expect("validation failed");
//! ```

use std::collections::HashMap;
use std::env;
use std::fmt;
use std::str::FromStr;

/// Error containing all validation failures.
#[derive(Debug, PartialEq)]
pub struct ValidationError {
    pub errors: Vec<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} validation error(s):", self.errors.len())?;
        for e in &self.errors {
            writeln!(f, "  - {}", e)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Field type specification.
#[derive(Debug, Clone)]
pub enum FieldType {
    Str,
    Int,
    Float,
    Bool,
    Url,
}

/// Configuration for a single environment variable.
#[derive(Debug, Clone)]
pub struct FieldSpec {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default: Option<String>,
    pub choices: Option<Vec<String>>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

/// Schema builder for environment variable validation.
#[derive(Debug, Default, Clone)]
pub struct Schema {
    fields: Vec<FieldSpec>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn string(self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder::new(self, name, FieldType::Str)
    }

    pub fn integer(self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder::new(self, name, FieldType::Int)
    }

    pub fn float(self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder::new(self, name, FieldType::Float)
    }

    pub fn boolean(self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder::new(self, name, FieldType::Bool)
    }

    pub fn url(self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder::new(self, name, FieldType::Url)
    }

    /// Returns the number of fields registered in the schema.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Validate environment variables and return a map of parsed values.
    pub fn validate(&self) -> Result<HashMap<String, EnvValue>, ValidationError> {
        self.validate_from(None)
    }

    /// Validate from a custom source map.
    pub fn validate_from(
        &self,
        source: Option<&HashMap<String, String>>,
    ) -> Result<HashMap<String, EnvValue>, ValidationError> {
        let mut errors = Vec::new();
        let mut result = HashMap::new();

        for spec in &self.fields {
            let raw = match source {
                Some(map) => map.get(&spec.name).cloned(),
                None => env::var(&spec.name).ok(),
            };

            let raw = match raw {
                Some(v) if !v.is_empty() => v,
                _ => {
                    if let Some(ref default) = spec.default {
                        default.clone()
                    } else if spec.required {
                        errors.push(format!("missing required variable: {}", spec.name));
                        continue;
                    } else {
                        continue;
                    }
                }
            };

            if let Some(ref choices) = spec.choices {
                if !choices.contains(&raw) {
                    errors.push(format!(
                        "{} must be one of {:?}, got '{}'",
                        spec.name, choices, raw
                    ));
                    continue;
                }
            }

            match parse_value(&raw, &spec.field_type) {
                Ok(val) => {
                    if let Some(msg) = check_constraints(spec, &val, &raw) {
                        errors.push(format!("{}: {}", spec.name, msg));
                    } else {
                        result.insert(spec.name.clone(), val);
                    }
                }
                Err(msg) => errors.push(format!("{}: {}", spec.name, msg)),
            }
        }

        if errors.is_empty() {
            Ok(result)
        } else {
            Err(ValidationError { errors })
        }
    }
}

/// Builder for field specifications.
pub struct FieldSpecBuilder {
    schema: Schema,
    spec: FieldSpec,
}

impl FieldSpecBuilder {
    fn new(schema: Schema, name: &str, field_type: FieldType) -> Self {
        Self {
            schema,
            spec: FieldSpec {
                name: name.to_string(),
                field_type,
                required: true,
                default: None,
                choices: None,
                min: None,
                max: None,
                min_length: None,
                max_length: None,
            },
        }
    }

    pub fn required(mut self, r: bool) -> Self {
        self.spec.required = r;
        self
    }

    pub fn default_value(mut self, v: &str) -> Self {
        self.spec.default = Some(v.to_string());
        self
    }

    pub fn choices(mut self, c: &[&str]) -> Self {
        self.spec.choices = Some(c.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Set the minimum allowed numeric value (for `integer` and `float` fields).
    pub fn min(mut self, v: f64) -> Self {
        self.spec.min = Some(v);
        self
    }

    /// Set the maximum allowed numeric value (for `integer` and `float` fields).
    pub fn max(mut self, v: f64) -> Self {
        self.spec.max = Some(v);
        self
    }

    /// Set the minimum allowed string length (for `string` and `url` fields).
    pub fn min_length(mut self, n: usize) -> Self {
        self.spec.min_length = Some(n);
        self
    }

    /// Set the maximum allowed string length (for `string` and `url` fields).
    pub fn max_length(mut self, n: usize) -> Self {
        self.spec.max_length = Some(n);
        self
    }

    pub fn build(mut self) -> Schema {
        self.schema.fields.push(self.spec);
        self.schema
    }
}

fn check_constraints(spec: &FieldSpec, value: &EnvValue, raw: &str) -> Option<String> {
    // Numeric range checks
    let numeric = match value {
        EnvValue::Int(n) => Some(*n as f64),
        EnvValue::Float(f) => Some(*f),
        _ => None,
    };
    if let Some(n) = numeric {
        if let Some(min) = spec.min {
            if n < min {
                return Some(format!("{} is less than minimum {}", raw, min));
            }
        }
        if let Some(max) = spec.max {
            if n > max {
                return Some(format!("{} is greater than maximum {}", raw, max));
            }
        }
    }

    // Length checks for strings (also applies to url since url is stored as Str)
    if matches!(value, EnvValue::Str(_)) {
        let len = raw.chars().count();
        if let Some(min) = spec.min_length {
            if len < min {
                return Some(format!(
                    "length {} is less than minimum length {}",
                    len, min
                ));
            }
        }
        if let Some(max) = spec.max_length {
            if len > max {
                return Some(format!(
                    "length {} is greater than maximum length {}",
                    len, max
                ));
            }
        }
    }

    None
}

/// A parsed environment variable value.
#[derive(Debug, Clone)]
pub enum EnvValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl EnvValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            EnvValue::Str(s) => Some(s),
            _ => None,
        }
    }

    /// Return the value rendered as an owned `String`, regardless of variant.
    ///
    /// Equivalent to `format!("{}", self)` but more discoverable.
    pub fn as_string(&self) -> String {
        self.to_string()
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            EnvValue::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            EnvValue::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            EnvValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
}

impl fmt::Display for EnvValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvValue::Str(s) => write!(f, "{}", s),
            EnvValue::Int(n) => write!(f, "{}", n),
            EnvValue::Float(v) => write!(f, "{}", v),
            EnvValue::Bool(b) => write!(f, "{}", b),
        }
    }
}

impl PartialEq for EnvValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EnvValue::Str(a), EnvValue::Str(b)) => a == b,
            (EnvValue::Int(a), EnvValue::Int(b)) => a == b,
            (EnvValue::Float(a), EnvValue::Float(b)) => a.to_bits() == b.to_bits(),
            (EnvValue::Bool(a), EnvValue::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl From<String> for EnvValue {
    fn from(s: String) -> Self {
        EnvValue::Str(s)
    }
}

impl From<&str> for EnvValue {
    fn from(s: &str) -> Self {
        EnvValue::Str(s.to_string())
    }
}

impl From<i64> for EnvValue {
    fn from(n: i64) -> Self {
        EnvValue::Int(n)
    }
}

impl From<f64> for EnvValue {
    fn from(v: f64) -> Self {
        EnvValue::Float(v)
    }
}

impl From<bool> for EnvValue {
    fn from(b: bool) -> Self {
        EnvValue::Bool(b)
    }
}

fn parse_value(raw: &str, field_type: &FieldType) -> Result<EnvValue, String> {
    match field_type {
        FieldType::Str => Ok(EnvValue::Str(raw.to_string())),
        FieldType::Int => i64::from_str(raw)
            .map(EnvValue::Int)
            .map_err(|_| format!("cannot convert '{}' to int", raw)),
        FieldType::Float => f64::from_str(raw)
            .map(EnvValue::Float)
            .map_err(|_| format!("cannot convert '{}' to float", raw)),
        FieldType::Bool => match raw.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Ok(EnvValue::Bool(true)),
            "false" | "0" | "no" | "off" => Ok(EnvValue::Bool(false)),
            _ => Err(format!("cannot convert '{}' to bool", raw)),
        },
        FieldType::Url => {
            if raw.starts_with("http://") || raw.starts_with("https://") {
                Ok(EnvValue::Str(raw.to_string()))
            } else {
                Err(format!("'{}' is not a valid URL", raw))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_required_field_present() {
        let src = source(&[("HOST", "localhost")]);
        let result = Schema::new()
            .string("HOST")
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["HOST"].as_str().unwrap(), "localhost");
    }

    #[test]
    fn test_required_field_missing() {
        let src = source(&[]);
        let err = Schema::new()
            .string("HOST")
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert_eq!(err.errors.len(), 1);
        assert!(err.errors[0].contains("missing required variable"));
    }

    #[test]
    fn test_optional_field_missing() {
        let src = source(&[]);
        let result = Schema::new()
            .string("HOST")
            .required(false)
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert!(!result.contains_key("HOST"));
    }

    #[test]
    fn test_default_value() {
        let src = source(&[]);
        let result = Schema::new()
            .integer("PORT")
            .default_value("3000")
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["PORT"].as_int().unwrap(), 3000);
    }

    #[test]
    fn test_integer_parsing() {
        let src = source(&[("PORT", "8080")]);
        let result = Schema::new()
            .integer("PORT")
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["PORT"].as_int().unwrap(), 8080);
    }

    #[test]
    fn test_integer_invalid() {
        let src = source(&[("PORT", "abc")]);
        let err = Schema::new()
            .integer("PORT")
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("cannot convert"));
    }

    #[test]
    fn test_float_parsing() {
        let src = source(&[("RATE", "3.14")]);
        let result = Schema::new()
            .float("RATE")
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert!((result["RATE"].as_float().unwrap() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn test_boolean_variants() {
        for (input, expected) in &[
            ("true", true),
            ("1", true),
            ("yes", true),
            ("on", true),
            ("false", false),
            ("0", false),
            ("no", false),
            ("off", false),
        ] {
            let src = source(&[("FLAG", input)]);
            let result = Schema::new()
                .boolean("FLAG")
                .build()
                .validate_from(Some(&src))
                .unwrap();
            assert_eq!(result["FLAG"].as_bool().unwrap(), *expected);
        }
    }

    #[test]
    fn test_boolean_invalid() {
        let src = source(&[("FLAG", "maybe")]);
        let err = Schema::new()
            .boolean("FLAG")
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("cannot convert"));
    }

    #[test]
    fn test_url_valid() {
        let src = source(&[("API", "https://example.com")]);
        let result = Schema::new()
            .url("API")
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["API"].as_str().unwrap(), "https://example.com");
    }

    #[test]
    fn test_url_invalid() {
        let src = source(&[("API", "not-a-url")]);
        let err = Schema::new()
            .url("API")
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("not a valid URL"));
    }

    #[test]
    fn test_choices_valid() {
        let src = source(&[("ENV", "production")]);
        let result = Schema::new()
            .string("ENV")
            .choices(&["development", "staging", "production"])
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["ENV"].as_str().unwrap(), "production");
    }

    #[test]
    fn test_choices_invalid() {
        let src = source(&[("ENV", "testing")]);
        let err = Schema::new()
            .string("ENV")
            .choices(&["development", "staging", "production"])
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("must be one of"));
    }

    #[test]
    fn test_multiple_errors() {
        let src = source(&[]);
        let err = Schema::new()
            .string("A")
            .build()
            .string("B")
            .build()
            .string("C")
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert_eq!(err.errors.len(), 3);
    }

    #[test]
    fn test_multiple_fields_valid() {
        let src = source(&[("HOST", "localhost"), ("PORT", "8080"), ("DEBUG", "true")]);
        let result = Schema::new()
            .string("HOST")
            .build()
            .integer("PORT")
            .build()
            .boolean("DEBUG")
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["HOST"].as_str().unwrap(), "localhost");
        assert_eq!(result["PORT"].as_int().unwrap(), 8080);
        assert_eq!(result["DEBUG"].as_bool().unwrap(), true);
    }

    #[test]
    fn test_empty_value_treated_as_missing() {
        let src = source(&[("HOST", "")]);
        let err = Schema::new()
            .string("HOST")
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("missing required variable"));
    }

    #[test]
    fn test_display_validation_error() {
        let err = ValidationError {
            errors: vec!["error one".to_string(), "error two".to_string()],
        };
        let display = format!("{}", err);
        assert!(display.contains("2 validation error(s)"));
        assert!(display.contains("error one"));
        assert!(display.contains("error two"));
    }

    #[test]
    fn test_env_value_display() {
        assert_eq!(format!("{}", EnvValue::Str("hello".into())), "hello");
        assert_eq!(format!("{}", EnvValue::Int(42)), "42");
        assert_eq!(format!("{}", EnvValue::Float(3.14)), "3.14");
        assert_eq!(format!("{}", EnvValue::Bool(true)), "true");
    }

    #[test]
    fn test_env_value_partial_eq() {
        assert_eq!(EnvValue::Str("a".into()), EnvValue::Str("a".into()));
        assert_ne!(EnvValue::Str("a".into()), EnvValue::Str("b".into()));
        assert_eq!(EnvValue::Int(1), EnvValue::Int(1));
        assert_ne!(EnvValue::Int(1), EnvValue::Int(2));
        assert_eq!(EnvValue::Float(1.5), EnvValue::Float(1.5));
        assert_ne!(EnvValue::Float(1.5), EnvValue::Float(2.5));
        assert_eq!(EnvValue::Bool(true), EnvValue::Bool(true));
        assert_ne!(EnvValue::Bool(true), EnvValue::Bool(false));
        assert_ne!(EnvValue::Int(1), EnvValue::Str("1".into()));
    }

    #[test]
    fn test_env_value_from_impls() {
        assert_eq!(EnvValue::from("hello"), EnvValue::Str("hello".into()));
        assert_eq!(EnvValue::from("hello".to_string()), EnvValue::Str("hello".into()));
        assert_eq!(EnvValue::from(42i64), EnvValue::Int(42));
        assert_eq!(EnvValue::from(3.14f64), EnvValue::Float(3.14));
        assert_eq!(EnvValue::from(true), EnvValue::Bool(true));
    }

    #[test]
    fn test_schema_clone() {
        let src = source(&[("HOST", "localhost")]);
        let schema = Schema::new().string("HOST").build();
        let schema2 = schema.clone();
        let r1 = schema.validate_from(Some(&src)).unwrap();
        let r2 = schema2.validate_from(Some(&src)).unwrap();
        assert_eq!(r1["HOST"], r2["HOST"]);
    }

    #[test]
    fn test_validation_error_partial_eq() {
        let e1 = ValidationError { errors: vec!["a".into()] };
        let e2 = ValidationError { errors: vec!["a".into()] };
        let e3 = ValidationError { errors: vec!["b".into()] };
        assert_eq!(e1, e2);
        assert_ne!(e1, e3);
    }

    #[test]
    fn test_min_int() {
        let src = source(&[("PORT", "100")]);
        let err = Schema::new()
            .integer("PORT")
            .min(1024.0)
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("less than minimum"));
    }

    #[test]
    fn test_max_int() {
        let src = source(&[("PORT", "70000")]);
        let err = Schema::new()
            .integer("PORT")
            .max(65535.0)
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("greater than maximum"));
    }

    #[test]
    fn test_int_within_range() {
        let src = source(&[("PORT", "8080")]);
        let result = Schema::new()
            .integer("PORT")
            .min(1024.0)
            .max(65535.0)
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["PORT"].as_int().unwrap(), 8080);
    }

    #[test]
    fn test_min_float() {
        let src = source(&[("RATE", "0.05")]);
        let err = Schema::new()
            .float("RATE")
            .min(0.1)
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("less than minimum"));
    }

    #[test]
    fn test_min_length_string() {
        let src = source(&[("PASSWORD", "abc")]);
        let err = Schema::new()
            .string("PASSWORD")
            .min_length(8)
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("less than minimum length"));
    }

    #[test]
    fn test_max_length_string() {
        let src = source(&[("CODE", "abcdefghij")]);
        let err = Schema::new()
            .string("CODE")
            .max_length(5)
            .build()
            .validate_from(Some(&src))
            .unwrap_err();
        assert!(err.errors[0].contains("greater than maximum length"));
    }

    #[test]
    fn test_string_length_within_range() {
        let src = source(&[("CODE", "abcdef")]);
        let result = Schema::new()
            .string("CODE")
            .min_length(3)
            .max_length(10)
            .build()
            .validate_from(Some(&src))
            .unwrap();
        assert_eq!(result["CODE"].as_str().unwrap(), "abcdef");
    }

    #[test]
    fn test_field_count() {
        let schema = Schema::new()
            .string("A")
            .build()
            .integer("B")
            .build()
            .boolean("C")
            .build();
        assert_eq!(schema.field_count(), 3);
    }

    #[test]
    fn test_env_value_as_string() {
        assert_eq!(EnvValue::Str("hello".into()).as_string(), "hello");
        assert_eq!(EnvValue::Int(42).as_string(), "42");
        assert_eq!(EnvValue::Bool(true).as_string(), "true");
    }
}
