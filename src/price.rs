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

