use crate::Opt;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

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

    // 比较同名文件
    for entry in files1 {
        let file_name = entry.file_name().unwrap().to_str().unwrap();
        if files2_set.contains(&file_name.to_string()) {
            let path1 = entry.clone();
            let path2 = Path::new(&dir2);
            let path2 = path2.join(file_name);

            if !compare_files(path1.as_path(), path2.as_path())
                .await
                .unwrap()
            {
                println!("Files differ: {:?}", file_name);
            }
        }
    }
}

// 比较两个文件的内容
async fn compare_files(path1: &Path, path2: &Path) -> io::Result<bool> {
    let mut file1 = fs::File::open(path1)?;
    let mut file2 = fs::File::open(path2)?;

    let mut contents1 = Vec::new();
    let mut contents2 = Vec::new();

    file1.read_to_end(&mut contents1)?;
    file2.read_to_end(&mut contents2)?;

    Ok(contents1 == contents2)
}
