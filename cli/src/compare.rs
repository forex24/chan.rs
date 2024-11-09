use crate::Opt;
use csv::ReaderBuilder;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, Read};
use std::path::Path; // 添加 CSV 读取器

pub async fn parse(opt: &Opt) {
    let dir1 = opt.from.clone().unwrap(); // 第一个目录
    let dir2 = opt.to.clone().unwrap(); // 第二个目录

    // 获取第一个目录的文件列表
    let files1 = fs::read_dir(dir1)
        .unwrap()
        .filter_map(|entry: Result<_, _>| entry.ok().map(|e| e.path()))
        .collect::<Vec<_>>();

    // 获取第二个目录的文件列表
    let files2 = fs::read_dir(dir2.clone())
        .unwrap()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    // 创建一个哈希集来存储第二个目录的文件名
    let mut files2_set = HashSet::new();
    for entry in &files2 {
        files2_set.insert(entry.file_name().to_str().unwrap().to_string());
    }

    // 定义文件名与比较字段的映射
    let mut field_map: HashMap<&str, Vec<&str>> = HashMap::new();
    field_map.insert("kline_list.csv", vec!["begin_time", "end_time"]);

    field_map.insert(
        "bi_list.csv",
        vec!["begin_time", "dir", "high", "low", "is_sure"],
    );
    field_map.insert(
        "bs_point_history.csv",
        vec!["begin_time", "bsp_type", "is_sure", "relate_bsp1"],
    );
    field_map.insert(
        "bs_point_lst.csv",
        vec!["begin_time", "dir", "high", "low", "fx"],
    );

    field_map.insert(
        "seg_bs_point_lst.csv",
        vec!["begin_time", "dir", "high", "low", "fx"],
    );
    field_map.insert(
        "seg_list.csv",
        vec!["begin_time", "dir", "high", "low", "is_sure", "zs_count"],
    );
    field_map.insert(
        "seg_seg_list.csv",
        vec!["begin_time", "dir", "high", "low", "is_sure", "zs_count"],
    );
    field_map.insert(
        "seg_zs_list.csv",
        vec![
            "begin_time",
            "high",
            "low",
            "peak_high",
            "peak_low",
            "is_sure",
        ],
    );
    field_map.insert(
        "zs_list.csv",
        vec![
            "begin_time",
            "high",
            "low",
            "peak_high",
            "peak_low",
            "is_sure",
        ],
    );

    // 比较同名文件
    for entry in files1 {
        let file_name = entry.file_name().unwrap().to_str().unwrap();
        if files2_set.contains(file_name) {
            let path1 = entry.clone();
            let path2 = Path::new(&dir2).join(file_name);

            println!("Comapred file:{}", file_name);
            // 根据文件名获取要比较的字段列表
            let compare_fields = field_map
                .get(file_name)
                .map(|v| v.as_slice())
                .unwrap_or(&[]); // 使用 `map` 创建更长生命周期的引用

            if !compare_files(path1.as_path(), path2.as_path(), compare_fields)
                .await
                .unwrap()
            {
                println!("Files differ: {:?}", file_name);
            }
        }
    }
    println!("Compare Ok");
}

// 比较两个文件的内容
async fn compare_files(path1: &Path, path2: &Path, compare_fields: &[&str]) -> io::Result<bool> {
    let file1 = fs::File::open(path1)?;
    let file2 = fs::File::open(path2)?;

    let mut rdr1 = ReaderBuilder::new().from_reader(file1);
    let mut rdr2 = ReaderBuilder::new().from_reader(file2);

    let mut values1 = Vec::new();
    let mut values2 = Vec::new();

    // 如果没有指定比较字段，则提取所有字段
    if compare_fields.is_empty() {
        // 提取第一个文件的所有字段
        values1.extend(extract_all_fields(&mut rdr1)); // 使用辅助函数提取所有字段

        // 提取第二个文件的所有字段
        values2.extend(extract_all_fields(&mut rdr2)); // 使用辅助函数提取所有字段
    } else {
        // 提取指定比较字段的值
        for field in compare_fields {
            // 提取第一个文件的字段值
            values1.push(extract_field_value(&mut rdr1, field));
            // 提取第二个文件的字段值
            values2.push(extract_field_value(&mut rdr2, field));
        }
    }

    Ok(values1 == values2) // 比较提取的字段值
}

// Helper function to extract field value from a CSV reader
fn extract_field_value<R: Read>(rdr: &mut csv::Reader<R>, field: &str) -> String {
    if let Some(record) = rdr.records().filter_map(Result::ok).next() {
        if let Some(index) = record.iter().position(|f| f == field) {
            return record.get(index).unwrap().to_string().trim().to_string();
        }
    }
    String::new() // 如果字段不存在，返回空字符串
}

// Helper function to extract all fields from a CSV reader
fn extract_all_fields<R: Read>(rdr: &mut csv::Reader<R>) -> Vec<String> {
    rdr.records()
        .filter_map(Result::ok)
        .next()
        .map(|record| {
            record
                .iter()
                .map(|field| field.to_string().trim().to_string())
                .collect::<Vec<String>>() // Specify the type to collect into a Vec<String>
        })
        .unwrap_or_else(Vec::new) // Return an empty Vec if no records are found
}
