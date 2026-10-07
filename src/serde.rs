use core::fmt;
use core::marker::PhantomData;

use serde::de::{self, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::patch_field::PatchField;

struct PatchFieldVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for PatchFieldVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = PatchField<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("null or a value")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PatchField::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(PatchField::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(PatchField::Value)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.visit_some(deserializer)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_char<E>(self, value: char) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::deserialize(value.into_deserializer()).map(PatchField::Value)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        T::deserialize(de::value::SeqAccessDeserializer::new(seq)).map(PatchField::Value)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        T::deserialize(de::value::MapAccessDeserializer::new(map)).map(PatchField::Value)
    }
}

impl<T: Serialize> Serialize for PatchField<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            PatchField::Value(v) => serializer.serialize_some(v),
            PatchField::Absent | PatchField::Null => serializer.serialize_none(),
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for PatchField<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(PatchFieldVisitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::string::{String, ToString};

    #[derive(Debug, Deserialize, PartialEq)]
    struct Payload {
        #[serde(default)]
        description: PatchField<String>,
    }

    // Serialization

    #[test]
    fn serializes_like_option() {
        assert_eq!(
            serde_json::to_string(&PatchField::Value("Tomato")).unwrap(),
            r#""Tomato""#
        );
        assert_eq!(
            serde_json::to_string(&PatchField::<String>::Null).unwrap(),
            "null"
        );
        assert_eq!(
            serde_json::to_string(&PatchField::<String>::Absent).unwrap(),
            "null"
        );
    }

    #[test]
    fn skip_serializing_if_omits_absent_but_keeps_null() {
        #[derive(Serialize)]
        struct Response {
            #[serde(skip_serializing_if = "PatchField::is_absent")]
            name: PatchField<String>,
            #[serde(skip_serializing_if = "PatchField::is_absent")]
            nickname: PatchField<String>,
            #[serde(skip_serializing_if = "PatchField::is_absent")]
            description: PatchField<String>,
        }

        let json = serde_json::to_string(&Response {
            description: PatchField::Absent,
            nickname: PatchField::Null,
            name: PatchField::Value("Dr.".to_string()),
        })
        .unwrap();

        assert_eq!(json, r#"{"name":"Dr.","nickname":null}"#);
    }

    // Deserialization

    #[test]
    fn missing_field_with_default_deserializes_as_absent() {
        let payload: Payload = serde_json::from_str("{}").unwrap();

        assert_eq!(payload.description, PatchField::Absent);
    }

    #[test]
    fn null_field_deserializes_as_null() {
        let payload: Payload = serde_json::from_str(r#"{"description": null}"#).unwrap();

        assert_eq!(payload.description, PatchField::Null);
    }

    #[test]
    fn value_field_deserializes_as_value() {
        let payload: Payload = serde_json::from_str(r#"{"description": "Tomato"}"#).unwrap();

        assert_eq!(payload.description, PatchField::Value("Tomato".to_string()));
    }

    #[test]
    fn missing_field_without_default_is_an_error() {
        #[derive(Debug, Deserialize)]
        struct MissingDefault {
            #[allow(dead_code)]
            name: PatchField<String>,
        }

        let err = serde_json::from_str::<MissingDefault>("{}").unwrap_err();

        assert!(
            err.to_string().contains("missing field `name`"),
            "expected a missing-field error, got: {err}"
        );
    }
}
