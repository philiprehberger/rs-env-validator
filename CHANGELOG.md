# Changelog

## 0.3.9 (2026-03-31)

- Standardize README to 3-badge format with emoji Support section
- Update CI checkout action to v5 for Node.js 24 compatibility

## 0.3.8 (2026-03-27)

- Add GitHub issue templates, PR template, and dependabot configuration
- Update README badges and add Support section

## 0.3.7 (2026-03-22)

- Fix README and CHANGELOG compliance

## 0.3.6 (2026-03-17)

- Add crate-level documentation with usage examples

## 0.3.5 (2026-03-17)

- Add readme, rust-version, documentation to Cargo.toml
- Add Development section to README

## 0.3.4 (2026-03-16)

- Update install snippet to use full version

## 0.3.3 (2026-03-16)

- Add README badges
- Synchronize version across Cargo.toml, README, and CHANGELOG

## 0.3.0 (2026-03-13)

- Add `Display` impl for `EnvValue` — renders inner value as string
- Add `PartialEq` impl for `EnvValue` — bitwise comparison for floats
- Add `From<String>`, `From<&str>`, `From<i64>`, `From<f64>`, `From<bool>` impls for `EnvValue`
- Add `Clone` derive for `Schema` — allows reusing schema definitions
- Add `PartialEq` derive for `ValidationError` — cleaner test assertions

## 0.2.0 (2026-03-12)

- Remove unsafe code from builder pattern — replaced raw pointer usage with safe owned-self pattern
- Add `float()` builder method for float environment variables
- Add `url()` builder method for URL environment variables
- Add comprehensive test suite covering all field types, defaults, choices, and error handling

## 0.1.0 (2026-03-09)

- Initial release
