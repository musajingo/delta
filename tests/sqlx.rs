#![cfg(feature = "sqlx")]

use field_delta::{Delta, UnchangedDeltaError};
use sqlx::{Connection, Encode, Postgres, SqliteConnection, Type, postgres::PgArgumentBuffer};

#[test]
fn postgres_encoding_preserves_values_nulls_and_errors() {
    let mut plain = PgArgumentBuffer::default();
    let mut wrapped = PgArgumentBuffer::default();
    assert!(
        !<String as Encode<Postgres>>::encode("hello".into(), &mut plain)
            .unwrap()
            .is_null()
    );
    assert!(
        !<Delta<String> as Encode<Postgres>>::encode(Delta::Set("hello".into()), &mut wrapped,)
            .unwrap()
            .is_null()
    );
    assert_eq!(&plain[..], &wrapped[..]);

    let mut borrowed = PgArgumentBuffer::default();
    let value = Delta::Set("hello".to_owned());
    assert!(
        !<Delta<String> as Encode<Postgres>>::encode_by_ref(&value, &mut borrowed,)
            .unwrap()
            .is_null()
    );
    assert_eq!(&plain[..], &borrowed[..]);
    assert_eq!(
        <Delta<String> as Encode<Postgres>>::size_hint(&value),
        <String as Encode<Postgres>>::size_hint(&"hello".to_owned()),
    );
    assert_eq!(
        <Delta<String> as Type<Postgres>>::type_info(),
        <String as Type<Postgres>>::type_info(),
    );

    let mut buffer = PgArgumentBuffer::default();
    assert!(
        <Delta<String> as Encode<Postgres>>::encode(Delta::Clear, &mut buffer,)
            .unwrap()
            .is_null()
    );
    assert!(buffer.is_empty());
    assert_eq!(
        <Delta<String> as Encode<Postgres>>::produces(&Delta::Clear),
        <Option<String> as Encode<Postgres>>::produces(&None),
    );

    for borrowed in [false, true] {
        let mut buffer = PgArgumentBuffer::default();
        let unchanged = Delta::<String>::Unchanged;
        let result = if borrowed {
            <Delta<String> as Encode<Postgres>>::encode_by_ref(&unchanged, &mut buffer)
        } else {
            <Delta<String> as Encode<Postgres>>::encode(unchanged, &mut buffer)
        };
        let error = result.err().expect("Unchanged must fail encoding");
        assert!(error.downcast_ref::<UnchangedDeltaError>().is_some());
        assert!(buffer.is_empty());
    }
}

#[tokio::test]
async fn sqlite_binds_owned_and_borrowed_deltas_and_rejects_unchanged() {
    let mut connection = SqliteConnection::connect("sqlite::memory:").await.unwrap();
    sqlx::query("CREATE TABLE users (id INTEGER PRIMARY KEY, nickname TEXT)")
        .execute(&mut connection)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users (id, nickname) VALUES (1, 'original')")
        .execute(&mut connection)
        .await
        .unwrap();

    for borrowed in [false, true] {
        for delta in [Delta::Set("updated".to_owned()), Delta::Clear] {
            let expected = delta.value().cloned();
            if borrowed {
                sqlx::query("UPDATE users SET nickname = ? WHERE id = 1")
                    .bind(&delta)
                    .execute(&mut connection)
                    .await
                    .unwrap();
            } else {
                sqlx::query("UPDATE users SET nickname = ? WHERE id = 1")
                    .bind(delta)
                    .execute(&mut connection)
                    .await
                    .unwrap();
            }
            let stored: Option<String> =
                sqlx::query_scalar("SELECT nickname FROM users WHERE id = 1")
                    .fetch_one(&mut connection)
                    .await
                    .unwrap();
            assert_eq!(stored, expected);
        }
    }

    sqlx::query("UPDATE users SET nickname = 'preserve' WHERE id = 1")
        .execute(&mut connection)
        .await
        .unwrap();
    let unchanged = Delta::<String>::Unchanged;
    let mut query = sqlx::query::<sqlx::Sqlite>("UPDATE users SET nickname = ? WHERE id = 1");
    let error = query.try_bind(&unchanged).unwrap_err();
    assert!(error.downcast_ref::<UnchangedDeltaError>().is_some());

    let error = sqlx::query("UPDATE users SET nickname = ? WHERE id = 1")
        .bind(unchanged)
        .execute(&mut connection)
        .await
        .unwrap_err();
    match error {
        sqlx::Error::Encode(source) => {
            // Query::bind adds argument context and stores a string error.
            assert!(source.to_string().contains("cannot bind Delta::Unchanged"));
        }
        other => panic!("expected an encoding error, got {other}"),
    }
    let stored: String = sqlx::query_scalar("SELECT nickname FROM users WHERE id = 1")
        .fetch_one(&mut connection)
        .await
        .unwrap();
    assert_eq!(stored, "preserve");
}
