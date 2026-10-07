/// A three-state field for PATCH semantics.
///
/// `PatchField<T>` distinguishes the three states a nullable field can have in
/// a PATCH request payload:
///
/// - [`PatchField::Absent`]: the field was omitted, so leave the stored value
///   unchanged.
/// - [`PatchField::Null`]: the field was sent as `null`, so clear the stored
///   value.
/// - [`PatchField::Value`]: the field was sent with a value, so update the
///   stored value.
///
/// This is useful for nullable database columns, where "not mentioned" and
/// "explicitly set to null" must be handled differently.
///
/// ## Serde
///
/// Every struct field using `PatchField<T>` must have `#[serde(default)]`.
/// This is what lets serde construct [`PatchField::Absent`] when the JSON key
/// is omitted.
///
/// ```rust
/// use patch_field::PatchField;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UpdatePayload {
///     #[serde(default)]
///     pub notes: PatchField<String>,
/// }
/// ```
///
/// If `#[serde(default)]` is forgotten, an omitted field fails deserialization
/// instead of becoming [`PatchField::Null`]. Failing loudly is intentional, it
/// prevents an omitted field from being mistaken for explicit `null`, which
/// would usually mean "clear this value".
///
/// ## Serialization
///
/// JSON has no representation for "absent" as a value, so serializing a bare
/// [`PatchField::Absent`] produces `null`, just like [`PatchField::Null`].
/// When serializing structs, use `skip_serializing_if` to omit absent fields:
///
/// ```rust
/// use patch_field::PatchField;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct UpdateResponse {
///     #[serde(skip_serializing_if = "PatchField::is_absent")]
///     pub notes: PatchField<String>,
/// }
/// ```
///
/// ## Example
///
/// ```rust
/// use patch_field::PatchField;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UpdatePayload {
///     #[serde(default)]
///     pub notes: PatchField<String>,
/// }
///
/// let absent: UpdatePayload = serde_json::from_str(r#"{}"#).unwrap();
/// assert_eq!(absent.notes, PatchField::Absent);
///
/// let null: UpdatePayload = serde_json::from_str(r#"{"notes": null}"#).unwrap();
/// assert_eq!(null.notes, PatchField::Null);
///
/// let value: UpdatePayload = serde_json::from_str(r#"{"notes": "hi"}"#).unwrap();
/// assert_eq!(value.notes, PatchField::Value("hi".to_string()));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum PatchField<T> {
    /// The field was not included in the request and should not be changed by
    /// the update operation.
    #[default]
    Absent,
    /// A nullable field explicitly set to `null`.
    Null,
    /// The field was set to a value other than `null`.
    Value(T),
}

impl<T> PatchField<T> {
    /// Returns `true` if the field was not sent in the request.
    #[inline]
    pub fn is_absent(&self) -> bool {
        matches!(self, PatchField::Absent)
    }

    /// Returns true if the field is set to something, even set to `null`.
    #[inline]
    pub fn is_present(&self) -> bool {
        matches!(self, PatchField::Null | PatchField::Value(_))
    }

    /// Returns `true` if the field was explicitly set to `null`.
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, PatchField::Null)
    }

    /// Returns `true` if the field has an actual value.
    #[inline]
    pub fn is_value(&self) -> bool {
        matches!(self, PatchField::Value(_))
    }

    /// Returns a reference to the value, if there is one.
    #[inline]
    pub fn value(&self) -> Option<&T> {
        match self {
            PatchField::Value(v) => Some(v),
            _ => None,
        }
    }

    /// Consumes the field and returns the value, if there is one.
    #[inline]
    pub fn into_value(self) -> Option<T> {
        match self {
            PatchField::Value(v) => Some(v),
            _ => None,
        }
    }

    /// Converts `&PatchField<T>` to `PatchField<&T>`.
    #[inline]
    pub fn as_ref(&self) -> PatchField<&T> {
        match self {
            PatchField::Absent => PatchField::Absent,
            PatchField::Null => PatchField::Null,
            PatchField::Value(v) => PatchField::Value(v),
        }
    }

    /// Converts `&mut PatchField<T>` to `PatchField<&mut T>`.
    #[inline]
    pub fn as_mut(&mut self) -> PatchField<&mut T> {
        match self {
            PatchField::Absent => PatchField::Absent,
            PatchField::Null => PatchField::Null,
            PatchField::Value(v) => PatchField::Value(v),
        }
    }

    /// Maps a `PatchField<T>` to a `PatchField<U>` by applying `f` to a
    /// contained value, leaving `Absent` and `Null` untouched.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> PatchField<U> {
        match self {
            PatchField::Absent => PatchField::Absent,
            PatchField::Null => PatchField::Null,
            PatchField::Value(v) => PatchField::Value(f(v)),
        }
    }
}

impl<T> From<Option<Option<T>>> for PatchField<T> {
    fn from(value: Option<Option<T>>) -> Self {
        match value {
            None => PatchField::Absent,
            Some(None) => PatchField::Null,
            Some(Some(v)) => PatchField::Value(v),
        }
    }
}

impl<T> From<PatchField<T>> for Option<Option<T>> {
    fn from(value: PatchField<T>) -> Self {
        match value {
            PatchField::Absent => None,
            PatchField::Null => Some(None),
            PatchField::Value(v) => Some(Some(v)),
        }
    }
}
