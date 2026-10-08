use alloc::boxed::Box;
use core::{error::Error, fmt};

use sqlx::{Database, Encode, Type, encode::IsNull, error::BoxDynError};

use crate::Delta;

/// An unchanged delta cannot be encoded as a SQL value.
///
/// Omit the column from the update or handle it with a condition in SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnchangedDeltaError;

impl fmt::Display for UnchangedDeltaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .write_str("cannot bind Delta::Unchanged, omit the column or guard the update in SQL")
    }
}

impl Error for UnchangedDeltaError {}

impl<T, DB> Type<DB> for Delta<T>
where
    DB: Database,
    T: Type<DB>,
{
    fn type_info() -> DB::TypeInfo {
        T::type_info()
    }

    fn compatible(ty: &DB::TypeInfo) -> bool {
        <Option<T> as Type<DB>>::compatible(ty)
    }
}

impl<'q, T, DB> Encode<'q, DB> for Delta<T>
where
    DB: Database,
    T: Encode<'q, DB> + Type<DB>,
{
    // Only the Set arm has an owned fast path; the other arms stay in
    // encode_by_ref so the two methods cannot drift apart.
    fn encode(self, buf: &mut DB::ArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match self {
            Self::Set(value) => value.encode(buf),
            other => other.encode_by_ref(buf),
        }
    }

    fn encode_by_ref(&self, buf: &mut DB::ArgumentBuffer) -> Result<IsNull, BoxDynError> {
        match self {
            Self::Unchanged => Err(Box::new(UnchangedDeltaError)),
            Self::Clear => Ok(IsNull::Yes),
            Self::Set(value) => value.encode_by_ref(buf),
        }
    }

    fn produces(&self) -> Option<DB::TypeInfo> {
        match self {
            Self::Set(value) => value.produces(),
            Self::Clear | Self::Unchanged => Some(T::type_info()),
        }
    }

    fn size_hint(&self) -> usize {
        match self {
            Self::Set(value) => value.size_hint(),
            Self::Clear | Self::Unchanged => 0,
        }
    }
}
