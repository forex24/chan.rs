use std::{env, path::PathBuf};

use chrono::{DateTime, Datelike, Duration, DurationRound, NaiveDate, Timelike, Utc};
use tokio::{
    fs::{File, OpenOptions},
    io::{self, AsyncWriteExt},
};
use tracing::error;

use crate::{period::Period, Opt};
use ctrader_rs::{
    credentials::{AccountCredentials, ApplicationCredentials},
    util::{download::Kline, download_asset},
    Error, Session,
};

const DOWNLOAD_DAYS: u64 = 30;

fn round_datetime(dt:DateTime<Utc>) -> DateTime<Utc> {
    dt.duration_trunc(Duration::minutes(1)).unwrap()
}

pub async fn download(opt: &Opt) {
    let days = opt.days.unwrap_or(DOWNLOAD_DAYS);
    let start_time = match opt.start {
        Some(ref start) => parse_timestr(start),
        None => round_datetime(Utc::now()) - Duration::days(days as i64),
    };
    let end_time = match opt.end {
        Some(ref end) => parse_timestr(end),
        None => round_datetime(Utc::now()),
    };

    let download_symbols = opt.symbol.split(',').collect::<Vec<_>>();

    let server_url = env::var("server_url").expect("miss server addr in env");
    let application_credentials = ApplicationCredentials::load_from_env();
    let account_credentials = AccountCredentials::load_from_env();
    let period = opt.period.unwrap_or(Period::M1) as i32;
    let r = run(
        application_credentials,
        account_credentials,
        &server_url,
        download_symbols,
        period,
        start_time,
        end_time,
        opt.output.clone(),
    )
    .await;

    if let Err(e) = r {
        error!("run error: {:?}", e)
    }
}

pub fn parse_timestr(timestr: &str) -> DateTime<Utc> {
    let dt = NaiveDate::parse_from_str(timestr, "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(dt, Utc);
    datetime
}

#[allow(clippy::too_many_arguments)]
async fn run(
    application_credentials: ApplicationCredentials,
    account_credentials: AccountCredentials,
    server_url: &str,
    download_symbols: Vec<&str>,
    period: i32,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    output: Option<PathBuf>,
) -> Result<(), Error> {
    let mut client = Session::builder()
        .set_application_credentials(application_credentials)
        .set_account_credentials(account_credentials)
        .set_url_string(server_url)
        .unwrap()
        .set_io_timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap();
    client.connect().await?;
    //let server_version = client.server_version();
    println!("server version:{}", client.server_version());
    //let _ = client.auth_application().await?;
    //let _ = client.auth_account().await?;
    let mut fname = output.unwrap_or(PathBuf::from(download_symbols.join("_")));
    fname.set_extension("csv");
    let mut csv_file = create_file(&fname).await?;
    for download_symbol in download_symbols {
        let v = download_asset(&client, download_symbol, period, &start_date, &end_date).await?;
        println!("{} v len={}", download_symbol, v.len());
        write_bar(v, download_symbol, &mut csv_file).await?;
    }
    csv_file.sync_all().await?;
    client.shutdown().await?;
    Ok(())
}

async fn create_file(csv_file_name: &PathBuf) -> Result<File, Error> {
    let csv_file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&csv_file_name)
        .await?;

    //csv_file
    //    .write_all(b"timestamp,symbol,open,close,high,low,vol\n")
    //    .await?;

    Ok(csv_file)
}

async fn write_bar(v: Vec<Kline>, _symbol: &str, csv_file: &mut File) -> io::Result<()> {
    //let wtr =  csv::WriterBuilder::new().has_headers(false);
    let mut buffer = String::new();
    for kline in &v {
        if kline.timestamp.year() != 2024 {
            println!("error timestamp:{}", kline.timestamp);
        }
        // match postgresql table struct
        let timestr = format!(
            "{:04}{:02}{:02}{:02}{:02}00000",
            kline.timestamp.year(),
            kline.timestamp.month(),
            kline.timestamp.day(),
            kline.timestamp.hour(),
            kline.timestamp.minute()
        );
        let content = format!(
            "{},{},{},{},{}\n",
            timestr,
            //symbol,
            kline.open,
            kline.high,
            kline.low,
            kline.close,
            //kline.vol
        );
        buffer.push_str(&content);
    }
    csv_file.write_all(buffer.as_bytes()).await?;
    Ok(())
}
