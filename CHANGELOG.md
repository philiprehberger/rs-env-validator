# Changelog

## 0.3.0

- Add `Display` impl for `EnvValue` — renders inner value as string
- Add `PartialEq` impl for `EnvValue` — bitwise comparison for floats
- Add `From<String>`, `From<&str>`, `From<i64>`, `From<f64>`, `From<bool>` impls for `EnvValue`
- Add `Clone` derive for `Schema` — allows reusing schema definitions
- Add `PartialEq` derive for `ValidationError` — cleaner test assertions

## 0.2.0

- Remove unsafe code from builder pattern — replaced raw pointer usage with safe owned-self pattern
- Add `float()` builder method for float environment variables
- Add `url()` builder method for URL environment variables
- Add comprehensive test suite covering all field types, defaults, choices, and error handling

## 0.1.0

- Initial release
