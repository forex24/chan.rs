import pandas as pd
from collections import defaultdict

def find_duplicates(csv_file):
    # Read the CSV file
    df = pd.read_csv(csv_file)
    
    # Convert time columns to datetime
    time_columns = ['begin_time', 'bi_begin_time', 'bi_end_time']
    for col in time_columns:
        df[col] = pd.to_datetime(df[col])
    
    # Group by begin_time
    groups = defaultdict(list)
    for idx, row in df.iterrows():
        groups[row['begin_time']].append(row)
    
    # Columns to compare
    compare_cols = ['bsp_type', 'is_buy', 'relate_bsp1', 'bi_idx', 'bi_begin_time', 'bi_end_time']
    #compare_cols = ['is_buy', 'relate_bsp1', 'bi_idx', 'bi_begin_time', 'bi_end_time']
    #compare_cols = ['is_buy', 'bi_idx', 'bi_begin_time', 'bi_end_time']
    # Find inconsistent entries
    inconsistent = []
    for time, rows in groups.items():
        if len(rows) > 1:
            # Convert rows to list of tuples with only the comparison columns
            row_values = [tuple(row[compare_cols]) for row in rows]
            # If there are different values in the comparison columns
            if len(set(row_values)) > 1:
                inconsistent.extend([row for row in rows])
    
    if inconsistent:
        # Create DataFrame from inconsistent rows
        result_df = pd.DataFrame(inconsistent)
        # Sort by begin_time
        result_df = result_df.sort_values('begin_time')
        
        # Save to new CSV file
        output_file = csv_file.replace('.csv', '_inconsistent.csv')
        result_df.to_csv(output_file, index=False)
        print(f"Found {len(result_df)} inconsistent rows. Results saved to {output_file}")
        
        # Print the inconsistent rows
        print("\nInconsistent rows:")
        print(result_df.to_string())
    else:
        print("No inconsistencies found.")

if __name__ == "__main__":
    import sys
    if len(sys.argv) != 2:
        print("Usage: python find_no_dup.py <csv_file>")
        sys.exit(1)
    
    csv_file = sys.argv[1]
    find_duplicates(csv_file)
