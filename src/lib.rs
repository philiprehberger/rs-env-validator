use std::collections::HashMap;
use std::env;
use std::fmt;
use std::str::FromStr;

/// Error containing all validation failures.
#[derive(Debug)]
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
}

/// Schema builder for environment variable validation.
#[derive(Debug, Default)]
pub struct Schema {
    fields: Vec<FieldSpec>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn string(mut self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder {
            schema: &mut self as *mut Schema,
            spec: FieldSpec {
                name: name.to_string(),
                field_type: FieldType::Str,
                required: true,
                default: None,
                choices: None,
            },
        }
    }

    pub fn integer(mut self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder {
            schema: &mut self as *mut Schema,
            spec: FieldSpec {
                name: name.to_string(),
                field_type: FieldType::Int,
                required: true,
                default: None,
                choices: None,
            },
        }
    }

    pub fn boolean(mut self, name: &str) -> FieldSpecBuilder {
        FieldSpecBuilder {
            schema: &mut self as *mut Schema,
            spec: FieldSpec {
                name: name.to_string(),
                field_type: FieldType::Bool,
                required: true,
                default: None,
                choices: None,
            },
        }
    }

    fn add_field(&mut self, spec: FieldSpec) {
        self.fields.push(spec);
    }

    /// Validate environment variables and return a map of parsed values.
    pub fn validate(&self) -> Result<HashMap<String, EnvValue>, ValidationError> {
        self.validate_from(None)
    }

    /// Validate from a custom source map.
    pub fn validate_from(&self, source: Option<&HashMap<String, String>>) -> Result<HashMap<String, EnvValue>, ValidationError> {
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
                    errors.push(format!("{} must be one of {:?}, got '{}'", spec.name, choices, raw));
                    continue;
                }
            }

            match parse_value(&raw, &spec.field_type) {
                Ok(val) => { result.insert(spec.name.clone(), val); }
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
    schema: *mut Schema,
    spec: FieldSpec,
}

impl FieldSpecBuilder {
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

    pub fn build(self) -> Schema {
        unsafe {
            (*self.schema).add_field(self.spec);
            std::ptr::read(self.schema)
        }
    }
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
