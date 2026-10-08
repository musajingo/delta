#![cfg(feature = "utoipa")]

use patch_field::PatchField;
use serde_json::json;
use utoipa::{OpenApi, PartialSchema, ToSchema};

const DESCRIPTION: &str = "omit to leave unchanged, `null` clears the stored value.";

#[test]
fn primitive_fields_are_nullable() {
    let schema = serde_json::to_value(PatchField::<String>::schema()).unwrap();
    assert_eq!(
        schema["anyOf"],
        json!([{"type": "string"}, {"type": "null"}])
    );
    assert_eq!(schema["description"], DESCRIPTION);

    let schema = serde_json::to_value(PatchField::<i32>::schema()).unwrap();
    assert_eq!(schema["anyOf"][0]["type"], "integer");
    assert_eq!(schema["anyOf"][0]["format"], "int32");
    assert_eq!(schema["anyOf"][1], json!({"type": "null"}));
}

#[derive(ToSchema)]
#[allow(dead_code)]
struct Address {
    street: String,
}

#[derive(ToSchema)]
#[allow(dead_code)]
struct Profile {
    address: Address,
}

#[derive(ToSchema)]
#[allow(dead_code)]
struct Patch {
    #[serde(default, skip_serializing_if = "PatchField::is_absent")]
    nickname: PatchField<String>,
    #[serde(default)]
    age: PatchField<i32>,
    #[serde(default)]
    profile: PatchField<Profile>,
    // A nullable value is still required unless omission is permitted.
    required: PatchField<bool>,
}

#[derive(OpenApi)]
#[openapi(components(schemas(Patch)))]
struct Api;

#[test]
fn openapi_preserves_optional_fields_and_nested_components() {
    let document = serde_json::to_value(Api::openapi()).unwrap();
    let schemas = &document["components"]["schemas"];
    let patch = &schemas["Patch"];
    assert_eq!(patch["required"], json!(["required"]));

    for (field, name) in [
        ("nickname", "PatchField_String"),
        ("age", "PatchField_i32"),
        ("profile", "PatchField_Profile"),
    ] {
        assert_eq!(
            patch["properties"][field]["$ref"],
            format!("#/components/schemas/{name}")
        );
        assert!(schemas.get(name).is_some());
    }

    assert_eq!(schemas["PatchField_String"]["anyOf"][0]["type"], "string");
    assert_eq!(schemas["PatchField_i32"]["anyOf"][0]["type"], "integer");

    // A type reachable only through PatchField is still a named component,
    // and so are its own dependencies. Primitives stay inline-only.
    assert!(schemas.get("Profile").is_some());
    assert!(schemas.get("Address").is_some());
    assert!(schemas.get("String").is_none());
    assert!(schemas.get("i32").is_none());
    assert_eq!(schemas["Address"]["properties"]["street"]["type"], "string");
    assert_eq!(
        schemas["PatchField_Profile"]["anyOf"][0]["properties"]["address"]["$ref"],
        "#/components/schemas/Address"
    );
    assert_eq!(
        schemas["PatchField_Profile"]["anyOf"][1],
        json!({"type": "null"})
    );
}

#[cfg(feature = "serde")]
#[test]
fn documented_payload_retains_all_three_states() {
    #[derive(serde::Serialize, serde::Deserialize, ToSchema)]
    struct UpdateUser {
        #[serde(default, skip_serializing_if = "PatchField::is_absent")]
        nickname: PatchField<String>,
    }

    for (input, expected) in [
        ("{}", PatchField::Absent),
        (r#"{"nickname":null}"#, PatchField::Null),
        (r#"{"nickname":"musa"}"#, PatchField::Value("musa".into())),
    ] {
        let payload: UpdateUser = serde_json::from_str(input).unwrap();
        assert_eq!(payload.nickname, expected);
        assert_eq!(serde_json::to_string(&payload).unwrap(), input);
    }

    let schema = serde_json::to_value(UpdateUser::schema()).unwrap();
    assert!(schema.get("required").is_none());
}
