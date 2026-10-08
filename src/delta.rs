/// Describes a change to a field: leave it unchanged, clear it, or set a
/// value.
///
/// In a PATCH request payload, the three states map onto a nullable field:
///
/// - [`Delta::Unchanged`]: the field was omitted, so leave the stored value
///   unchanged.
/// - [`Delta::Clear`]: the field was sent as `null`, so clear the stored
///   value.
/// - [`Delta::Set`]: the field was sent with a value, so update the stored
///   value.
///
/// This is useful for nullable database columns, where "not mentioned" and
/// "explicitly set to null" must be handled differently.
///
/// ## Serde
///
/// Every struct field using `Delta<T>` must have `#[serde(default)]`.
/// This is what lets serde construct [`Delta::Unchanged`] when the JSON key
/// is omitted.
///
/// ```rust
/// use delta::Delta;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UpdatePayload {
///     #[serde(default)]
///     pub notes: Delta<String>,
/// }
/// ```
///
/// If `#[serde(default)]` is forgotten, an omitted field fails deserialization
/// instead of becoming [`Delta::Clear`]. Failing loudly is intentional, it
/// prevents an omitted field from being mistaken for explicit `null`, which
/// would usually mean "clear this value".
///
/// ## Serialization
///
/// JSON has no representation for an omitted field as a value, so serializing
/// a bare [`Delta::Unchanged`] produces `null`, just like [`Delta::Clear`].
/// When serializing structs, use `skip_serializing_if` to omit unchanged
/// fields:
///
/// ```rust
/// use delta::Delta;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct UpdateResponse {
///     #[serde(skip_serializing_if = "Delta::is_unchanged")]
///     pub notes: Delta<String>,
/// }
/// ```
///
/// ## Example
///
/// ```rust
/// use delta::Delta;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct UpdatePayload {
///     #[serde(default)]
///     pub notes: Delta<String>,
/// }
///
/// let unchanged: UpdatePayload = serde_json::from_str(r#"{}"#).unwrap();
/// assert_eq!(unchanged.notes, Delta::Unchanged);
///
/// let clear: UpdatePayload = serde_json::from_str(r#"{"notes": null}"#).unwrap();
/// assert_eq!(clear.notes, Delta::Clear);
///
/// let set: UpdatePayload = serde_json::from_str(r#"{"notes": "hi"}"#).unwrap();
/// assert_eq!(set.notes, Delta::Set("hi".to_string()));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Delta<T> {
    /// The field was not included in the request; leave the stored value
    /// unchanged.
    #[default]
    Unchanged,
    /// The field was explicitly set to `null`; clear the stored value.
    Clear,
    /// The field was sent with a value; set the stored value to it.
    Set(T),
}

impl<T> Delta<T> {
    /// Returns `true` if the field was not sent in the request.
    #[inline]
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Delta::Unchanged)
    }

    /// Returns `true` if the field was sent, even if it was sent as `null`.
    #[inline]
    pub fn is_changed(&self) -> bool {
        matches!(self, Delta::Clear | Delta::Set(_))
    }

    /// Returns `true` if the field was explicitly set to `null`.
    #[inline]
    pub fn is_clear(&self) -> bool {
        matches!(self, Delta::Clear)
    }

    /// Returns `true` if the field was sent with a value.
    #[inline]
    pub fn is_set(&self) -> bool {
        matches!(self, Delta::Set(_))
    }

    /// Returns a reference to the value, if one was set.
    #[inline]
    pub fn value(&self) -> Option<&T> {
        match self {
            Delta::Set(v) => Some(v),
            _ => None,
        }
    }

    /// Consumes the delta and returns the value, if one was set.
    #[inline]
    pub fn into_value(self) -> Option<T> {
        match self {
            Delta::Set(v) => Some(v),
            _ => None,
        }
    }

    /// Converts `&Delta<T>` to `Delta<&T>`.
    #[inline]
    pub fn as_ref(&self) -> Delta<&T> {
        match self {
            Delta::Unchanged => Delta::Unchanged,
            Delta::Clear => Delta::Clear,
            Delta::Set(v) => Delta::Set(v),
        }
    }

    /// Converts `&mut Delta<T>` to `Delta<&mut T>`.
    #[inline]
    pub fn as_mut(&mut self) -> Delta<&mut T> {
        match self {
            Delta::Unchanged => Delta::Unchanged,
            Delta::Clear => Delta::Clear,
            Delta::Set(v) => Delta::Set(v),
        }
    }

    /// Maps a `Delta<T>` to a `Delta<U>` by applying `f` to a set value,
    /// leaving `Unchanged` and `Clear` untouched.
    #[inline]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Delta<U> {
        match self {
            Delta::Unchanged => Delta::Unchanged,
            Delta::Clear => Delta::Clear,
            Delta::Set(v) => Delta::Set(f(v)),
        }
    }
}

impl<T> From<Option<Option<T>>> for Delta<T> {
    fn from(value: Option<Option<T>>) -> Self {
        match value {
            None => Delta::Unchanged,
            Some(None) => Delta::Clear,
            Some(Some(v)) => Delta::Set(v),
        }
    }
}

impl<T> From<Delta<T>> for Option<Option<T>> {
    fn from(value: Delta<T>) -> Self {
        match value {
            Delta::Unchanged => None,
            Delta::Clear => Some(None),
            Delta::Set(v) => Some(Some(v)),
        }
    }
}
