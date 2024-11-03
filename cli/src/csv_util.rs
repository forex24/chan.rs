use chrono::prelude::*;
use czsc::Kline;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

fn load_datetime_bar(csv: &str) -> Vec<Kline> {
    let mut bars: Vec<Kline> = Vec::with_capacity(100000);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(csv.as_bytes());
    let mut line_count = 0;
    for record in reader.records() {
        let record = record.unwrap();
        let timestr: &str = AsRef::<str>::as_ref(&record[0]);
        let mut timestr = timestr.to_string();
        timestr.truncate(12);
        let dt_result = NaiveDateTime::parse_from_str(&timestr, "%Y%m%d%H%M");
        if dt_result.is_err() {
            println!(
                "parse cvs error on line count {}, str:{} {:?}",
                line_count, timestr, dt_result
            );
            line_count += 1;
            continue;
        }
        let datetime = Utc.from_utc_datetime(&dt_result.unwrap()); //.with_timezone(&Utc);
        let open = AsRef::<str>::as_ref(&record[1]).parse::<f64>().unwrap();
        let high = AsRef::<str>::as_ref(&record[2]).parse::<f64>().unwrap();
        let low = AsRef::<str>::as_ref(&record[3]).parse::<f64>().unwrap();
        let close = AsRef::<str>::as_ref(&record[4]).parse::<f64>().unwrap();
        //let vol = AsRef::<str>::as_ref(&record[5]).parse::<f64>().unwrap();
        let bar = Kline::new(datetime, open, high, low, close, 0.0f64);
        bars.push(bar);
        line_count += 1;
    }
    bars
}

pub fn read_kline_from_csv(path: &Path) -> Vec<Kline> {
    let display = path.display();
    let mut file = match File::open(path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };
    let mut s = String::new();
    if let Err(why) = file.read_to_string(&mut s) {
        panic!("couldn't open {}: {}", display, why)
    };

    load_datetime_bar(&s)
}
