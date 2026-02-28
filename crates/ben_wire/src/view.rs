use crate::error::ViewError;
use crate::schema::Schema;
use crate::slot::SlotValue;

pub trait BenView: Sized {
    const EVT_HASH: [u8; 32];

    const FIELD_NAMES: &'static [&'static str];

    const FIELD_COUNT: usize;

    fn from_row(schema: &Schema, row: &[SlotValue]) -> Result<Self, ViewError>;
}

pub fn view<T>(schema: &Schema, row: &[SlotValue]) -> Result<T, ViewError>
where
    T: BenView,
{
    if schema.evt_hash != T::EVT_HASH {
        return Err(ViewError::EvtHashMismatch {
            expected: T::EVT_HASH,
            got: schema.evt_hash,
        });
    }

    if schema.fields_len() != T::FIELD_COUNT {
        return Err(ViewError::FieldCountMismatch {
            expected: T::FIELD_COUNT,
            got: schema.fields_len(),
        });
    }

    if row.len() != T::FIELD_COUNT {
        return Err(ViewError::FieldCountMismatch {
            expected: T::FIELD_COUNT,
            got: row.len(),
        });
    }

    #[cfg(debug_assertions)]
    {
        for (index, (field, &expected_name)) in
            schema.fields.iter().zip(T::FIELD_NAMES.iter()).enumerate()
        {
            if field.name != expected_name {
                return Err(ViewError::FieldNameMismatch {
                    index,
                    expected: expected_name,
                    got: field.name.clone(),
                });
            }
        }
    }

    T::from_row(schema, row)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Field, FieldType, Schema};
    use crate::slot::SlotValue;

    #[derive(Debug, PartialEq, Eq)]
    struct TestEvent {
        id: i64,
        name: String,
    }

    impl BenView for TestEvent {
        const EVT_HASH: [u8; 32] = [0x11; 32];
        const FIELD_NAMES: &'static [&'static str] = &["id", "name"];
        const FIELD_COUNT: usize = 2;

        fn from_row(_schema: &Schema, row: &[SlotValue]) -> Result<Self, ViewError> {
            if row.len() != Self::FIELD_COUNT {
                return Err(ViewError::FieldCountMismatch {
                    expected: Self::FIELD_COUNT,
                    got: row.len(),
                });
            }

            let id = match row[0] {
                SlotValue::I64(v) => v,
                _ => return Err(ViewError::TypeMismatch { field: "id" }),
            };

            let name = match &row[1] {
                SlotValue::Str(s) => s.clone().to_string(),
                _ => return Err(ViewError::TypeMismatch { field: "name" }),
            };

            Ok(TestEvent { id, name })
        }
    }

    fn make_test_schema() -> Schema {
        Schema {
            event: "test_event".to_string(),
            version: 1,
            evt_hash: TestEvent::EVT_HASH,
            fields: vec![
                Field {
                    name: "id".to_string(),
                    ty: FieldType::UInt64,
                    nullable: false,
                },
                Field {
                    name: "id".to_string(),
                    ty: FieldType::UInt64,
                    nullable: false,
                },
            ],
        }
    }

    #[test]
    fn view_successfully_deserializes_row_into_struct() {
        let schema = make_test_schema();
        let row = vec![SlotValue::I64(42), SlotValue::Str("dozer")];

        let ev = view::<TestEvent>(&schema, &row).expect("view should succeed");

        assert_eq!(
            ev,
            TestEvent {
                id: 42,
                name: "dozer".to_string()
            }
        );
    }

    #[test]
    fn view_rejects_mismatched_evt_hash() {
        let mut schema = make_test_schema();
        schema.evt_hash = [0x22; 32]; // wrong hash

        let row = vec![SlotValue::I64(42), SlotValue::Str("dozer")];

        let err = view::<TestEvent>(&schema, &row).expect_err("view should fail");

        match err {
            ViewError::EvtHashMismatch { expected, got } => {
                assert_eq!(expected, TestEvent::EVT_HASH);
                assert_eq!(got, [0x22; 32]);
            }
            other => panic!("expected EvtHashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn view_rejects_wrong_field_count() {
        let schema = make_test_schema();
        let row = vec![SlotValue::I64(42)]; // only 1 field, should be 2

        let err = view::<TestEvent>(&schema, &row).expect_err("view should fail");

        match err {
            ViewError::FieldCountMismatch { expected, got } => {
                assert_eq!(expected, TestEvent::FIELD_COUNT);
                assert_eq!(got, 1);
            }
            other => panic!("expected FieldCountMismatch, got {other:?}"),
        }
    }
}
