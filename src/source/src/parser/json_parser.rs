// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use risingwave_common::array::Op;
use risingwave_common::error::ErrorCode::ProtocolError;
use risingwave_common::error::{Result, RwError};
use risingwave_common::types::Datum;
use serde_json::Value;

use crate::parser::common::json_parse_value;
use crate::{Event, SourceColumnDesc, SourceParser};

/// Parser for JSON format
#[derive(Debug)]
pub struct JSONParser;

impl SourceParser for JSONParser {
    fn parse(&self, payload: &[u8], columns: &[SourceColumnDesc]) -> Result<Event> {
        let value: Value = serde_json::from_slice(payload)
            .map_err(|e| RwError::from(ProtocolError(e.to_string())))?;

        Ok(Event {
            ops: vec![Op::Insert],
            rows: vec![columns
                .iter()
                .map(|column| {
                    if column.skip_parse {
                        None
                    } else {
                        let v = json_parse_value(column, value.get(&column.name)).ok();
                        tracing::error!("{:?}",v);
                        tracing::error!("{:?}",column);
                        v
                    }
                })
                .collect::<Vec<Datum>>()],
        })
    }
}

#[cfg(test)]
mod tests {
    use risingwave_common::catalog::ColumnId;
    use risingwave_common::types::{DataType, ScalarImpl};
    use risingwave_expr::vector_op::cast::{str_to_date, str_to_timestamp};

    use crate::{JSONParser, SourceColumnDesc, SourceParser};

    #[test]
    fn test_json_parser() {
        let parser = JSONParser {};
        let payload = r#"{"i32":1,"bool":true,"i16":1,"i64":12345678,"f32":1.23,"f64":1.2345,"varchar":"varchar","date":"2021-01-01","timestamp":"2021-01-01 16:06:12.269"}"#.as_bytes();
        let descs = vec![
            SourceColumnDesc::new_atomic(
                "i32".to_string(),
                DataType::Int32,
                ColumnId::from(0),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "bool".to_string(),
                DataType::Boolean,
                ColumnId::from(2),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "i16".to_string(),
                DataType::Int16,
                ColumnId::from(3),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "i64".to_string(),
                DataType::Int64,
                ColumnId::from(4),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "f32".to_string(),
                DataType::Float32,
                ColumnId::from(5),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "f64".to_string(),
                DataType::Float64,
                ColumnId::from(6),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "varchar".to_string(),
                DataType::Varchar,
                ColumnId::from(7),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "date".to_string(),
                DataType::Date,
                ColumnId::from(8),
                false,
            ),
            SourceColumnDesc::new_atomic(
                "timestamp".to_string(),
                DataType::Timestamp,
                ColumnId::from(9),
                false,
            ),
        ];

        let result = parser.parse(payload, &descs);
        assert!(result.is_ok());
        let event = result.unwrap();
        let row = event.rows.first().unwrap();
        assert_eq!(row.len(), descs.len());
        assert!(row[0].eq(&Some(ScalarImpl::Int32(1))));
        assert!(row[1].eq(&Some(ScalarImpl::Bool(true))));
        assert!(row[2].eq(&Some(ScalarImpl::Int16(1))));
        assert!(row[3].eq(&Some(ScalarImpl::Int64(12345678))));
        assert!(row[4].eq(&Some(ScalarImpl::Float32(1.23.into()))));
        assert!(row[5].eq(&Some(ScalarImpl::Float64(1.2345.into()))));
        assert!(row[6].eq(&Some(ScalarImpl::Utf8("varchar".to_string()))));
        assert!(row[7].eq(&Some(ScalarImpl::NaiveDate(
            str_to_date("2021-01-01").unwrap()
        ))));
        assert!(row[8].eq(&Some(ScalarImpl::NaiveDateTime(
            str_to_timestamp("2021-01-01 16:06:12.269").unwrap()
        ))));

        let payload = r#"{"i32":1}"#.as_bytes();
        let result = parser.parse(payload, &descs);
        assert!(result.is_ok());
        let event = result.unwrap();
        let row = event.rows.first().unwrap();
        assert_eq!(row.len(), descs.len());
        assert!(row[0].eq(&Some(ScalarImpl::Int32(1))));
        assert!(row[1].eq(&None));

        let payload = r#"{"i32:1}"#.as_bytes();
        let result = parser.parse(payload, &descs);
        assert!(result.is_err());
    }
}
