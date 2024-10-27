use crate::Common::func_util::str2float;
use crate::Common::types::SharedCell;
use crate::Common::CEnum::{DataField, KlType};
use crate::Common::ChanException::{CChanException, ErrCode};
use crate::DataAPI::CommonStockAPI::CCommonStockApi;
use crate::KLine::KLine_Unit::CKLineUnit;
use chrono::{NaiveDate, NaiveDateTime};
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

fn create_item_dict(
    data: &[String],
    column_name: &[DataField],
) -> Result<std::collections::HashMap<DataField, f64>, CChanException> {
    let mut result = std::collections::HashMap::new();
    for (i, field) in column_name.iter().enumerate() {
        let value = if *field == DataField::FieldTime {
            parse_time_column(&data[i])? as f64
        } else {
            str2float(&data[i])?
        };
        result.insert(*field, value);
    }
    Ok(result)
}

fn parse_time_column(inp: &str) -> Result<i64, CChanException> {
    let parse_result = match inp.len() {
        10 => NaiveDate::parse_from_str(inp, "%Y-%m-%d")
            .map(|date| date.and_hms_opt(0, 0, 0).unwrap()),
        17 => NaiveDateTime::parse_from_str(inp, "%Y%m%d%H%M%S000"),
        19 => NaiveDateTime::parse_from_str(inp, "%Y-%m-%d %H:%M:%S"),
        _ => {
            return Err(CChanException::new(
                &format!("unknown time column from csv:{}", inp),
                ErrCode::SrcDataFormatError,
            ))
        }
    };
    parse_result.map(|dt| dt.timestamp()).map_err(|_| {
        CChanException::new(
            &format!("Failed to parse time: {}", inp),
            ErrCode::SrcDataFormatError,
        )
    })
}

pub struct CsvApi {
    code: String,
    name: Option<String>,
    is_stock: Option<bool>,
    k_type: KlType,
    begin_date: Option<String>,
    end_date: Option<String>,
    autype: Option<String>,
    headers_exist: bool,
    columns: Vec<DataField>,
    time_column_idx: usize,
}

impl CCommonStockApi for CsvApi {
    fn new(
        code: String,
        k_type: KlType,
        begin_date: Option<String>,
        end_date: Option<String>,
        autype: Option<String>,
    ) -> Self {
        let columns = vec![
            DataField::FieldTime,
            DataField::FieldOpen,
            DataField::FieldHigh,
            DataField::FieldLow,
            DataField::FieldClose,
            DataField::FieldVolume,
        ];
        let time_column_idx = columns
            .iter()
            .position(|&r| r == DataField::FieldTime)
            .unwrap();
        let mut api = CsvApi {
            code,
            name: None,
            is_stock: None,
            k_type,
            begin_date,
            end_date,
            autype,
            headers_exist: true,
            columns,
            time_column_idx,
        };
        api.set_basic_info();
        api
    }

    fn get_kl_data(
        &self,
    ) -> Box<dyn Iterator<Item = Result<SharedCell<CKLineUnit>>, CChanException>> {
        let file_path = format!("/opt/data/raw_data/{}.csv", self.code);
        let file = match File::open(&file_path) {
            Ok(file) => file,
            Err(_) => {
                return Box::new(std::iter::once(Err(CChanException::new(
                    &format!("file not exist: {}", file_path),
                    ErrCode::SrcDataNotFound,
                ))))
            }
        };
        let reader = BufReader::new(file);
        let lines = reader.lines().enumerate();

        let headers_exist = self.headers_exist;
        let columns = self.columns.clone();
        let time_column_idx = self.time_column_idx;
        let begin_date = self.begin_date.clone();
        let end_date = self.end_date.clone();

        Box::new(lines.filter_map(move |(line_number, line_result)| {
            if headers_exist && line_number == 0 {
                return None;
            }
            let line = match line_result {
                Ok(line) => line,
                Err(_) => {
                    return Some(Err(CChanException::new(
                        "Error reading line from file".to_string(),
                        ErrCode::SrcDataFormatError,
                    )))
                }
            };
            let data: Vec<String> = line.split(',').map(String::from).collect();
            if data.len() != columns.len() {
                return Some(Err(CChanException::new(
                    &format!("file format error: {}", file_path),
                    ErrCode::SrcDataFormatError,
                )));
            }
            if let Some(ref begin) = begin_date {
                if &data[time_column_idx] < begin {
                    return None;
                }
            }
            if let Some(ref end) = end_date {
                if &data[time_column_idx] > end {
                    return None;
                }
            }
            match create_item_dict(&data, &columns) {
                Ok(dict) => Some(Ok(Rc::new(RefCell::new(
                    CKLineUnit::new(&dict, false).unwrap(),
                )))),
                Err(e) => Some(Err(e)),
            }
        }))
    }

    fn set_basic_info(&mut self) {
        // Implement if needed
    }

    fn do_init() {
        // Implement if needed
    }

    fn do_close() {
        // Implement if needed
    }
}
