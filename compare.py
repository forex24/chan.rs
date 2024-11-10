import pandas as pd
import os
import argparse
from typing import List, Dict
from datetime import datetime

# 定义要比较的列
COMPARE_COLS = {
    'kline_list.csv': ['begin_time', 'idx','dir', 'high', 'low', 'fx'],
    'bi_list.csv': ['begin_time', 'idx', 'dir', 'high', 'low', 'is_sure','seg_idx', 'parent_seg', 'begin_klc', 'end_klc', 'begin_val', 'end_val', 'klu_cnt', 'klc_cnt'],
    'seg_list.csv': ['begin_time', 'idx','dir', 'high', 'low', 'is_sure','start_bi_idx','end_bi_idx', 'zs_count', 'bi_count','reason'],
    'zs_list.csv': ['begin_time', 'high', 'low', 'peak_high', 'peak_low', 'is_sure', 'begin_bi_idx', 'bi_in', 'bi_out'],
    'bs_point_lst.csv': ['begin_time', 'bsp_type', 'bi_idx', 'bi_begin_time', 'bi_end_time'],
    'bs_point_history.csv':['begin_time', 'bsp_type'],
    
    'seg_seg_list.csv': ['begin_time', 'idx','dir', 'high', 'low', 'is_sure','zs_count', 'bi_count','reason'],
    'seg_zs_list.csv': ['begin_time', 'high', 'low', 'peak_high', 'peak_low', 'is_sure'],
    'seg_bs_point_lst.csv': ['begin_time', 'bsp_type', 'bi_begin_time', 'bi_end_time'],
    'seg_bs_point_history.csv':['begin_time', 'bsp_type'],
    
}

def get_compare_columns(filename: str) -> List[str]:
    """根据文件名返回需要比较的列"""
    return COMPARE_COLS.get(filename, [])

def normalize_datetime(val):
    """标准化时间格式"""
    if isinstance(val, str):
        val = val.strip()
        try:
            # 尝试不同的时间格式
            formats = [
                '%Y-%m-%d %H:%M',  # 2024-01-08 00:00
                '%Y/%m/%d %H:%M',  # 2024/01/08 00:00
                '%Y-%m-%d',        # 2024-01-08
                '%Y/%m/%d'         # 2024/01/08
            ]
            
            for fmt in formats:
                try:
                    dt = datetime.strptime(val, fmt)
                    # 如果原始字符串没有时间部分，或者时间是00:00，则只返回日期部分
                    if fmt in ['%Y-%m-%d', '%Y/%m/%d'] or dt.strftime('%H:%M') == '00:00':
                        return dt.strftime('%Y-%m-%d')
                    return dt.strftime('%Y-%m-%d %H:%M')
                except ValueError:
                    continue
        except Exception:
            pass
    return val

def clean_value(val):
    """清理值，对字符串进行trim处理"""
    if isinstance(val, str):
        return val.strip()
    return val

def compare_files(dir1: str, dir2: str):
    """比较两个目录下的同名文件"""
    if not os.path.exists(dir1):
        print(f"错误: 目录 {dir1} 不存在")
        return
    if not os.path.exists(dir2):
        print(f"错误: 目录 {dir2} 不存在")
        return
    
    files_to_compare = list(COMPARE_COLS.keys())
    
    for filename in files_to_compare:
        path1 = os.path.join(dir1, filename)
        path2 = os.path.join(dir2, filename)
        
        if not os.path.exists(path1):
            print(f"文件 {filename} 在 {dir1} 目录中不存在")
            continue
        if not os.path.exists(path2):
            print(f"文件 {filename} 在 {dir2} 目录中不存在")
            continue
            
        compare_cols = get_compare_columns(filename)
        try:
            df1 = pd.read_csv(path1)
            df2 = pd.read_csv(path2)
            
            # 确保两个DataFrame只包含需要比较的列
            df1 = df1[compare_cols]
            df2 = df2[compare_cols]
            
            # 标准化时间列
            time_cols = [col for col in compare_cols if 'time' in col.lower()]
            for col in time_cols:
                df1[col] = df1[col].apply(normalize_datetime)
                df2[col] = df2[col].apply(normalize_datetime)
            
            # 对其他字符串列进行trim处理
            non_time_cols = [col for col in compare_cols if 'time' not in col.lower()]
            for col in non_time_cols:
                if df1[col].dtype == 'object':
                    df1[col] = df1[col].apply(clean_value)
                    df2[col] = df2[col].apply(clean_value)
            
            if len(df1) != len(df2):
                print(f"{filename} 行数不同: {dir1}={len(df1)}, {dir2}={len(df2)}")
                continue
                
            differences_found = False
            for idx in range(len(df1)):
                row1 = df1.iloc[idx]
                row2 = df2.iloc[idx]
                
                # 逐列比较
                row_different = False
                diff_cols = []
                for col in compare_cols:
                    if row1[col] != row2[col]:
                        row_different = True
                        diff_cols.append(col)
                
                if row_different:
                    if not differences_found:
                        print(f"\n{filename} 中发现差异:")
                        differences_found = True
                    print(f"  第 {idx+1} 行:")
                    for col in diff_cols:
                        print(f"    列 {col}: {dir1}={row1[col]}, {dir2}={row2[col]}")
            
            if not differences_found:
                print(f"{filename} 比较完成: 内容相同")
                    
        except Exception as e:
            print(f"比较 {filename} 时发生错误: {str(e)}")

def main():
    parser = argparse.ArgumentParser(description='比较两个目录下的CSV文件')
    parser.add_argument('--dir1', default='output', help='第一个目录路径 (默认: output)')
    parser.add_argument('--dir2', default='python_result', help='第二个目录路径 (默认: python_result)')
    
    args = parser.parse_args()
    compare_files(args.dir1, args.dir2)

if __name__ == "__main__":
    main()
