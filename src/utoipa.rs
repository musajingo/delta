use alloc::{borrow::Cow, string::String, vec::Vec};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{RefOr, schema::Schema},
};

use crate::Delta;

// Type names utoipa always in-lines instead of registering as components.
// We don't register these as named schemas, it would add junk entries like
// `String` to the document
const INLINED_PRIMITIVES: &[&str] = &[
    "i8",
    "i16",
    "i32",
    "i64",
    "i128",
    "isize",
    "u8",
    "u16",
    "u32",
    "u64",
    "u128",
    "usize",
    "bool",
    "f32",
    "f64",
    "String",
    "str",
    "char",
    "TupleUnit",
];

// #[derive(ToSchema)] calls this for generic field types like `Delta<T>`,
// handing us T's schema. `__dev` is internal API, but it is the only hook
// utoipa offers.
impl<T: PartialSchema> utoipa::__dev::ComposeSchema for Delta<T> {
    fn compose(mut schemas: Vec<RefOr<Schema>>) -> RefOr<Schema> {
        utoipa::openapi::schema::AnyOfBuilder::new()
            .item(if schemas.is_empty() {
                T::schema()
            } else {
                schemas.remove(0)
            })
            .item(
                utoipa::openapi::schema::ObjectBuilder::new()
                    .schema_type(utoipa::openapi::schema::Type::Null),
            )
            .description(Some(
                "omit to leave unchanged, `null` clears the stored value.",
            ))
            .into()
    }
}

impl<T: ToSchema> ToSchema for Delta<T> {
    // Utoipa adds the `_T` suffix itself: this becomes `Delta_String`.
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("Delta")
    }

    fn schemas(schemas: &mut Vec<(String, RefOr<Schema>)>) {
        let name = T::name();
        if !INLINED_PRIMITIVES.contains(&name.as_ref()) {
            schemas.push((name.into_owned(), T::schema()));
        }
        T::schemas(schemas);
    }
}
