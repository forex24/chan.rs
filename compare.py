import pandas as pd
import os
import argparse
from typing import List, Dict
from datetime import datetime

# 定义要比较的列
COMPARE_COLS = {
    'kline_list.csv': ['begin_time', 'end_time', 'idx','dir', 'high', 'low', 'fx'],
    'bi_list.csv':['begin_time','end_time','idx','dir','high','low','is_sure','seg_idx','begin_klc','end_klc','begin_val','end_val','klu_cnt','klc_cnt','parent_seg_idx','parent_seg_dir'],
    'seg_list.csv':['begin_time','end_time','idx','dir','high','low','is_sure','start_bi_idx','end_bi_idx','zs_count','bi_count','reason','parent_seg_idx','parent_seg_dir'],
    'zs_list.csv':['begin_time','end_time','high','low','peak_high','peak_low','is_sure','begin_bi_idx','end_bi_idx','bi_in','bi_out','sub_zs_count'],
    'bs_point_lst.csv': ['begin_time', 'bsp_type', 'bi_idx', 'bi_begin_time', 'bi_end_time','relate_bsp1_time'],
    'bs_point_history.csv':['begin_time', 'bsp_type', 'is_buy', 'bi_idx', 'bi_begin_time','bi_end_time','relate_bsp1'],
    
    'seg_seg_list.csv':  ['begin_time', 'end_time','idx','dir', 'high', 'low', 'is_sure','start_seg_idx','end_seg_idx', 'zs_count', 'bi_count','reason'],
    'seg_zs_list.csv': ['begin_time', 'end_time','high', 'low', 'peak_high', 'peak_low', 'is_sure', 'begin_seg_idx', 'end_seg_idx','bi_in', 'bi_out'],
    'seg_bs_point_lst.csv': ['begin_time', 'bsp_type', 'seg_idx', 'bi_begin_time', 'bi_end_time'],
    'segseg_history.csv':['clock','end_bi_begin_klu_time','begin_time','end_time','idx','dir','high','low','is_sure','start_seg_idx','end_seg_idx','zs_count','bi_count','reason'],
    'seg_bs_point_history.csv':['begin_time','bsp_type','is_buy','relate_bsp1','seg_idx','bi_begin_time','bi_end_time']
}


def get_compare_columns(filename: str) -> List[str]:
    """根据文件名返回需要比较的列"""
    return COMPARE_COLS.get(filename, [])

def normalize_datetime(val):
    """标准化时间格式"""
    if isinstance(val, str):
        val = val.strip()
        # 移除 UTC 后缀
        val = val.replace(' UTC', '')
        try:
            # 尝试不同的时间格式
            formats = [
                '%Y-%m-%d %H:%M:%S',  # 2024-01-08 00:00:00
                '%Y-%m-%d %H:%M',     # 2024-01-08 00:00
                '%Y/%m/%d %H:%M',     # 2024/01/08 00:00
                '%Y-%m-%d',           # 2024-01-08
                '%Y/%m/%d'            # 2024/01/08
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

def normalize_bsp_type(val):
    """标准化 bsp_type 值，使得不同的分隔符表示相同的含义"""
    if isinstance(val, str):
        # 将逗号分隔改为下划线分隔
        return val.replace(',', '_')
    return val

def normalize_parent_seg_value(val):
    """标准化parent_seg值，使得整数和浮点数可以比较"""
    if pd.isna(val):  # 处理 nan 值
        return val
    if isinstance(val, float) and val.is_integer():
        return int(val)
    return val

def is_nan_equal(val1, val2):
    """比较两个值，特殊处理 nan 值"""
    if pd.isna(val1) and pd.isna(val2):
        return True
    return val1 == val2

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
            time_cols = [col for col in compare_cols if ('time' in col.lower()) or col.lower() == 'relate_bsp1' or col.lower() == 'clock']
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
                    val1 = row1[col]
                    val2 = row2[col]
                    
                    # 对不同类型的列进行特殊处理
                    if col in ['parent_seg_idx', 'parent_seg_dir', 'relate_bsp1', 'relate_bsp1_time']:  # 添加这些特殊列
                        if is_nan_equal(val1, val2):
                            continue
                    elif col == 'is_sure':
                        if isinstance(val1, str) and isinstance(val2, str):
                            val1 = val1.lower()
                            val2 = val2.lower()
                    elif col == 'bsp_type':
                        if isinstance(val1, str) and isinstance(val2, str):
                            val1 = normalize_bsp_type(val1)
                            val2 = normalize_bsp_type(val2)
                    else:
                        val1 = clean_value(val1)
                        val2 = clean_value(val2)
                    
                    if val1 != val2:
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
    if os.name == 'posix':
        root_directory = '/'
    # 对于Windows系统
    elif os.name == 'nt':
        root_directory = os.path.splitdrive(os.getcwd())[0] + '\\'
    default_path = os.path.join(root_directory, 'opt', 'data','dump_data')
    parser = argparse.ArgumentParser(description='比较两个目录下的CSV文件')
    parser.add_argument('--dir1', default='output', help='第一个目录路径 (默认: output)')
    parser.add_argument('--dir2', default='python_result', help='第二个目录路径 (默认: python_result)')
    parser.add_argument('--symbol', help='要比较的品种')
    
    args = parser.parse_args()

    if args.symbol is not None:
        compare_files(f'{args.symbol}_output', os.path.join(default_path, f'{args.symbol}'))
    else:
        compare_files(args.dir1, args.dir2)

if __name__ == "__main__":
    main()
