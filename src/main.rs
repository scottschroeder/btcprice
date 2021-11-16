use argparse::CliOpts;

mod argparse;
mod price {
    use serde::{Deserialize, Serialize};

    const PRICE_HISTORY_URL: &str = "https://api.blockchain.info/charts/market-price?format=csv";

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct BlockChainRecord {
        date: chrono::NaiveDate,
        price: f64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct QuickenCsv {
        symbol: String,
        price: f64,
        date: chrono::NaiveDate,
    }

    fn parse_date(s: &str) -> anyhow::Result<chrono::NaiveDate> {
        let date_part = s
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow::anyhow!("unable to extract date"))?;
        Ok(chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")?)
    }

    fn parse_price(s: &str) -> anyhow::Result<f64> {
        Ok(s.parse::<f64>()?)
    }

    pub fn download() -> anyhow::Result<Vec<BlockChainRecord>> {
        let response = reqwest::blocking::get(PRICE_HISTORY_URL)?;
        let mut rdr = csv::Reader::from_reader(response);
        rdr.set_headers(csv::StringRecord::from(vec!["date", "price"]));

        let mut prices = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let dt = parse_date(&record[0])?;
            let price = parse_price(&record[1])?;
            let pr = BlockChainRecord { date: dt, price };
            prices.push(pr);
        }

        Ok(prices)
    }

    pub fn into_quicken_data(symbol: &str, prices: &[BlockChainRecord]) -> Vec<QuickenCsv> {
        prices
            .iter()
            .map(|p| QuickenCsv {
                symbol: symbol.to_string(),
                price: p.price,
                date: p.date,
            })
            .collect()
    }

    pub fn write_records<W: std::io::Write>(w: W, data: &[QuickenCsv]) -> anyhow::Result<()> {
        let mut wtr = csv::WriterBuilder::new().from_writer(w);
        for d in data {
            wtr.write_record(&[
                d.symbol.as_str(),
                &d.date.format("%m/%d/%Y").to_string(),
                &format!("{:.06}", d.price),
            ])?;
        }
        Ok(())
    }

    /// btc record in 1/1000 units
    pub fn convert_to_mbtc(prices: &[QuickenCsv]) -> Vec<QuickenCsv> {
        prices
            .iter()
            .map(|q| QuickenCsv {
                symbol: "miliBTC".to_string(),
                price: q.price / 1000.0,
                date: q.date,
            })
            .collect()
    }
}

fn main() -> anyhow::Result<()> {
    color_backtrace::install();
    let args = argparse::get_args();
    setup_logger(args.verbose);
    // cli::setup_logger(args.occurrences_of("verbosity"));
    log::trace!("Args: {:?}", args);

    run(&args).map_err(|e| {
        log::error!("{}", e);
        e.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));
        anyhow::anyhow!("unrecoverable astrometry failure")
    })
}

fn run(args: &CliOpts) -> anyhow::Result<()> {
    match &args.subcmd {
        argparse::SubCommand::Test(_opts) => todo!("no test command"),
        argparse::SubCommand::Btc(_) => prices(false),
        argparse::SubCommand::MBtc(_) => prices(true),
    }
}

fn prices(convert_to_mbtc: bool) -> anyhow::Result<()> {
    let prices = price::download()?;
    let mut quicken_date = price::into_quicken_data("BTC", prices.as_slice());

    if convert_to_mbtc {
        quicken_date = price::convert_to_mbtc(&quicken_date)
    }

    let f = std::fs::File::create("out.csv")?;
    price::write_records(f, quicken_date.as_slice())?;
    Ok(())
}

pub fn setup_logger(level: u8) {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    let noisy_modules = &[
        "hyper",
        "mio",
        "tokio_core",
        "tokio_reactor",
        "tokio_threadpool",
        "fuse::request",
        "rusoto_core",
        "want",
        "tantivy",
    ];

    let log_level = match level {
        //0 => log::Level::Error,
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    if level > 1 && level < 4 {
        for module in noisy_modules {
            builder.filter_module(module, log::LevelFilter::Info);
        }
    }

    builder.filter_level(log_level);
    builder.format_timestamp_millis();
    //builder.format(|buf, record| writeln!(buf, "{}", record.args()));
    builder.init();
}
