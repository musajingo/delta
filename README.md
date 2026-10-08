# Delta

`Delta<T>` describes a change to a field: leave it unchanged, clear it, or
set a value.

```rust
pub enum Delta<T> {
    Unchanged,
    Clear,
    Set(T),
}
```

In a PATCH request body, the three states map onto JSON naturally:

```json
{}                   // omitted: leave the stored value unchanged
{ "notes": null }    // null: clear the stored value
{ "notes": "hi" }    // value: set the stored value
```

Plain `Option<T>` can represent `null` or a value, but not an omitted field.
`Delta<T>` keeps all three apart.

## Install

```toml
[dependencies]
delta = { git = "https://github.com/musamahmoudjingo/delta" }
```

The crate is `no_std`. The optional `utoipa` and `sqlx` features link `std`
through their dependencies. The `serde` feature is enabled by default.

```toml
[dependencies]
delta = { git = "https://github.com/musamahmoudjingo/delta", default-features = false }
```

Disable default features only if you do not need serde support.

## Serde

Always put `#[serde(default)]` on struct fields that use `Delta<T>`:

```rust
use delta::Delta;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UpdateUser {
    #[serde(default)]
    nickname: Delta<String>,
}
```

With `#[serde(default)]`, omitted fields become `Delta::Unchanged`:

```rust
use delta::Delta;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UpdateUser {
    #[serde(default)]
    nickname: Delta<String>,
}

let update: UpdateUser = serde_json::from_str("{}").unwrap();
assert_eq!(update.nickname, Delta::Unchanged);

let update: UpdateUser = serde_json::from_str(r#"{"nickname": null}"#).unwrap();
assert_eq!(update.nickname, Delta::Clear);

let update: UpdateUser = serde_json::from_str(r#"{"nickname": "musa"}"#).unwrap();
assert_eq!(update.nickname, Delta::Set("musa".to_string()));
```

If `#[serde(default)]` is forgotten, an omitted field fails deserialization.
This fail-loud behavior prevents an omitted field from accidentally being
treated as explicit `null`, which often means "clear this database column."

```rust
use delta::Delta;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MissingDefault {
    nickname: Delta<String>,
}

let err = serde_json::from_str::<MissingDefault>("{}").unwrap_err();
assert!(err.to_string().contains("missing field `nickname`"));
```

## Serialization

JSON has no standalone value for an omitted field, so serializing a bare
`Delta::Unchanged` produces `null`, the same as `Delta::Clear`:

```rust
use delta::Delta;

assert_eq!(serde_json::to_string(&Delta::Set("hi")).unwrap(), r#""hi""#);
assert_eq!(serde_json::to_string(&Delta::<String>::Clear).unwrap(), "null");
assert_eq!(serde_json::to_string(&Delta::<String>::Unchanged).unwrap(), "null");
```

When serializing a struct, use `skip_serializing_if` to omit unchanged fields
while keeping explicit `null` values:

```rust
use delta::Delta;
use serde::Serialize;

#[derive(Serialize)]
struct UpdateUser {
    #[serde(skip_serializing_if = "Delta::is_unchanged")]
    nickname: Delta<String>,
    #[serde(skip_serializing_if = "Delta::is_unchanged")]
    bio: Delta<String>,
}

let json = serde_json::to_string(&UpdateUser {
    nickname: Delta::Unchanged,
    bio: Delta::Clear,
})
.unwrap();

assert_eq!(json, r#"{"bio":null}"#);
```

## OpenAPI with utoipa

Enable the `utoipa` feature to use `Delta<T>` in `#[derive(ToSchema)]`
types:

```toml
[dependencies]
delta = { git = "https://github.com/musamahmoudjingo/delta", features = ["utoipa"] }
```

<!-- Not a doctest: needs the non-default `utoipa` feature. Covered by tests/utoipa.rs. -->

```rust,ignore
use delta::Delta;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
struct UpdateUser {
    /// The user's full name.
    #[serde(default)]
    name: Option<String>,

    /// The name shown to other users.
    #[serde(default)]
    nickname: Delta<String>,
}
```

## Database updates

Enable the optional `sqlx` feature to bind `Delta<T>` directly with sqlx 0.9.

```toml
[dependencies]
delta = { git = "https://github.com/musamahmoudjingo/delta", features = ["sqlx"] }
```

`Set(value)` binds the value. `Clear` binds SQL `NULL`. `Unchanged` fails
with `UnchangedDeltaError` instead of silently clearing the column, so use
`is_changed()` to decide which columns to update.

<!-- Not a doctest: needs the non-default sqlx feature and an application runtime. -->

```rust,ignore
use delta::Delta;
use sqlx::PgPool;

async fn update_nickname(
    pool: &PgPool,
    user_id: i64,
    nickname: &Delta<String>,
) -> Result<(), sqlx::Error> {
    if nickname.is_changed() {
        sqlx::query("UPDATE users SET nickname = $1 WHERE id = $2")
            .bind(nickname)
            .bind(user_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}
```

### Without the sqlx feature

Bind `is_changed()` and `value()` as a pair in a fixed query. The `CASE
WHEN` keeps the stored value when the field is unchanged.

<!-- Not a doctest: sqlx is an application dependency. -->

```rust,ignore
use delta::Delta;
use sqlx::PgPool;

async fn update_nickname(
    pool: &PgPool,
    user_id: i64,
    nickname: &Delta<String>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users
         SET nickname = CASE WHEN $1 THEN $2 ELSE nickname END
         WHERE id = $3",
    )
    .bind(nickname.is_changed())
    .bind(nickname.value())
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}
```

Or skip the query entirely when the field is unchanged. Inside the
`is_changed()` check, `value()` binds the new value for `Set` and `NULL`
for `Clear`.

```rust,ignore
use delta::Delta;
use sqlx::PgPool;

async fn update_nickname(
    pool: &PgPool,
    user_id: i64,
    nickname: &Delta<String>,
) -> Result<(), sqlx::Error> {
    if nickname.is_changed() {
        sqlx::query("UPDATE users SET nickname = $1 WHERE id = $2")
            .bind(nickname.value())
            .bind(user_id)
            .execute(pool)
            .await?;
    }
    Ok(())
}
```

> **Warning.** Never bind `value()` on its own. It returns `None` for both
> `Unchanged` and `Clear`, so the query would write `NULL` and clear fields
> the request never mentioned.

```rust,ignore
// Wrong. Unchanged becomes NULL and clears the column.
sqlx::query("UPDATE users SET nickname = $1 WHERE id = $2")
    .bind(nickname.value())
    .bind(user_id)
```
