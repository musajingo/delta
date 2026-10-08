# PatchField

`PatchField<T>` is a small enum for PATCH request payloads where a nullable
field needs three states:

```rust
pub enum PatchField<T> {
    Absent,
    Null,
    Value(T),
}
```

Those states map naturally to JSON PATCH-style update semantics:

```json
{}                   // absent: leave the stored value unchanged
{ "notes": null }    // null: clear the stored value
{ "notes": "hi" }    // value: update the stored value
```

Plain `Option<T>` can represent `null` or a value, but it cannot also represent
an omitted field. `PatchField<T>` keeps those cases separate.

## Install

```toml
[dependencies]
patch-field = "0.1"
```

The crate is `no_std`. The `serde` feature is enabled by default:

```toml
[dependencies]
patch-field = { version = "0.1", default-features = false }
```

Disable default features only if you do not need serde support.

## OpenAPI with utoipa

Enable the `utoipa` feature to use `PatchField<T>` in `#[derive(ToSchema)]`
types:

```toml
[dependencies]
patch-field = { version = "0.1", features = ["utoipa"] }
```

<!-- Not a doctest: needs the non-default `utoipa` feature. Covered by tests/utoipa.rs. -->

```rust,ignore
use patch_field::PatchField;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
struct UpdateUser {
    /// The user's full name.
    #[serde(default)]
    name: Option<String>, // absent = null/None

    /// The name shown to other users.
    #[serde(default)]
    nickname: PatchField<String>, // absent ≠ null/None
}
```

## Serde Usage

Always put `#[serde(default)]` on struct fields that use `PatchField<T>`:

```rust
use patch_field::PatchField;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UpdateUser {
    #[serde(default)]
    nickname: PatchField<String>,
}
```

With `#[serde(default)]`, omitted fields become `PatchField::Absent`:

```rust
use patch_field::PatchField;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UpdateUser {
    #[serde(default)]
    nickname: PatchField<String>,
}

let update: UpdateUser = serde_json::from_str("{}").unwrap();
assert_eq!(update.nickname, PatchField::Absent);

let update: UpdateUser = serde_json::from_str(r#"{"nickname": null}"#).unwrap();
assert_eq!(update.nickname, PatchField::Null);

let update: UpdateUser = serde_json::from_str(r#"{"nickname": "musa"}"#).unwrap();
assert_eq!(update.nickname, PatchField::Value("musa".to_string()));
```

If `#[serde(default)]` is forgotten, an omitted field fails deserialization
instead of becoming `PatchField::Null`. This fail-loud behavior prevents an
omitted field from accidentally being treated as explicit `null`, which often
means "clear this database column."

```rust
use patch_field::PatchField;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MissingDefault {
    nickname: PatchField<String>,
}

let err = serde_json::from_str::<MissingDefault>("{}").unwrap_err();
assert!(err.to_string().contains("missing field `nickname`"));
```

## Serialization

JSON has no standalone value for "absent", so serializing a bare
`PatchField::Absent` produces `null`, the same as `PatchField::Null`:

```rust
use patch_field::PatchField;

assert_eq!(serde_json::to_string(&PatchField::Value("hi")).unwrap(), r#""hi""#);
assert_eq!(serde_json::to_string(&PatchField::<String>::Null).unwrap(), "null");
assert_eq!(serde_json::to_string(&PatchField::<String>::Absent).unwrap(), "null");
```

When serializing a struct, use `skip_serializing_if` to omit absent fields while
keeping explicit `null` values:

```rust
use patch_field::PatchField;
use serde::Serialize;

#[derive(Serialize)]
struct UpdateUser {
    #[serde(skip_serializing_if = "PatchField::is_absent")]
    nickname: PatchField<String>,
    #[serde(skip_serializing_if = "PatchField::is_absent")]
    bio: PatchField<String>,
}

let json = serde_json::to_string(&UpdateUser {
    nickname: PatchField::Absent,
    bio: PatchField::Null,
})
.unwrap();

assert_eq!(json, r#"{"bio":null}"#);
```
