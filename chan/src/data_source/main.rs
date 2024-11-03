use arrow::record_batch::RecordBatch;
use clap::Parser;
use futures::stream::{self, StreamExt};
use parquet::file::reader::{FileReader, SerializedFileReader};
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;
use tokio::task;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional symbol to process specific parquet file
    #[arg(short, long)]
    symbol: Option<String>,

    /// Directory containing parquet files
    #[arg(short, long)]
    dir: Option<String>,
}

pub async fn read_parquet_files(
    dir_path: &str,
    symbol: Option<&str>,
) -> Result<Vec<RecordBatch>, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();

    if let Some(sym) = symbol {
        // 处理单个文件
        let file_path = format!("{}/{}.parquet", dir_path, sym);
        let path = PathBuf::from(file_path);
        if path.exists() {
            paths.push(path);
        } else {
            return Err(format!("File not found: {}.parquet", sym).into());
        }
    } else {
        // 处理目录下所有文件
        for entry in fs::read_dir(dir_path)? {
            let path = entry?.path();
            if path.extension().and_then(|s| s.to_str()) == Some("parquet") {
                paths.push(path);
            }
        }
    }

    // 并行处理文件
    let results: Vec<_> = stream::iter(paths)
        .map(|path| {
            let path = path.clone();
            task::spawn(async move { process_single_file(path).await })
        })
        .buffer_unwind(num_cpus::get()) // 限制并发数为 CPU 核心数
        .collect()
        .await;

    // 合并结果
    let mut all_batches = Vec::new();
    for result in results {
        match result? {
            Ok(mut batches) => all_batches.append(&mut batches),
            Err(e) => eprintln!("Error processing file: {}", e),
        }
    }

    Ok(all_batches)
}

async fn process_single_file(
    path: PathBuf,
) -> Result<Vec<RecordBatch>, Box<dyn std::error::Error>> {
    // 在单独的线程中处理 IO 密集型操作
    let batches = task::spawn_blocking(move || {
        let mut file_batches = Vec::new();
        let file = File::open(&path)?;
        let reader = SerializedFileReader::new(file)?;

        let iter = reader.get_row_iter(None)?;
        for record in iter {
            // 处理每个 RecordBatch
            // 这里需要根据实际数据结构进行转换
            // file_batches.push(record?);
        }

        Ok::<_, Box<dyn std::error::Error>>(file_batches)
    })
    .await??;

    Ok(batches)
}

// 使用示例
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 确定数据目录
    let data_dir = args
        .dir
        .or_else(|| env::var("ANALYSIS_DIR").ok())
        .unwrap_or_else(|| "/opt/data/raw_data".to_string());

    println!("Using data directory: {}", data_dir);

    // 读取文件
    let batches = read_parquet_files(&data_dir, args.symbol.as_deref()).await?;

    // 处理读取到的数据
    for batch in batches {
        // 处理每个 RecordBatch
    }

    Ok(())
}
