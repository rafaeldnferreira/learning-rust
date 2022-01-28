use std::collections::HashMap;
use std::fmt::{Debug};
use std::fs;
use std::io::{Error, ErrorKind};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use chrono;
use chrono::{DateTime, Local};

use jsonpath::Selector;
use serde_json::Value;

const JSON_PATH_MARKET_RAW_PRICE: &'static str = "$.quoteSummary.result[0].price.regularMarketPrice.raw";
const QUOTE_POOLING_TIME: u64 = 60 * 10;

///
/// Represents the value of an asset at a given point in time
///
#[derive(Debug, Clone)]
pub struct Quote {
    symbol: Box<str>,
    value: f64,
    pub timestamp: DateTime<Local>,
}

impl Quote {
    pub fn as_json(&self) -> String {
        format!("{{ \"timestamp\": \"{}\", \"symbol\": \"{}\", \"value\": {} }}", self.timestamp.to_rfc3339(), self.symbol, self.value)
    }
}

///
/// Capable of retrieving quote information and listing available symbols.
///
pub trait QuoteRetriever {
    fn find(&self, symbol: &str) -> Option<Quote>;

    fn available_quotes(&self) -> Vec<String>;
}

///
/// Provides the capability required to store quotes thru the lifecycle of the application.
///
pub struct QuoteService {
    quotes: Arc<RwLock<HashMap<Box<str>, Quote>>>,
}

impl QuoteService {
    pub fn new() -> QuoteService {
        let service = QuoteService { quotes: Arc::new(RwLock::new(HashMap::new())) };
        service.trigger_pooling();
        service
    }


    fn trigger_pooling(&self) {
        let quotes = Arc::clone(&self.quotes);

        thread::spawn(move || {
            let available_quotes = fs::read_to_string("symbols.txt")
                .expect("symbols.txt must be available.");

            let symbols: Vec<&str> = available_quotes.lines().collect();

            loop {
                for &symbol in &symbols {

                    match get_quote(symbol) {
                        Ok(quote) => {
                            println!("Retrieved quote for {}", symbol);

                            quotes.write()
                                .unwrap()
                                .insert(Box::from(symbol), quote);
                        }
                        Err(e) => { eprintln!("Unable to retrieve symbol: {}", e) }
                    }

                }

                thread::sleep(Duration::from_secs(QUOTE_POOLING_TIME));
            }
        });
    }


}

fn get_quote(symbol: &str) -> Result<Quote, Box<dyn std::error::Error>> {
    let json: Value = reqwest::blocking::get(format!("https://query1.finance.yahoo.com/v10/finance/quoteSummary/{}?formatted=true&modules=price", symbol))?
        .json()?;

    let value = extract_market_price(&json)?;

    Ok(Quote { symbol: Box::from(symbol), value, timestamp: chrono::offset::Local::now() })
}

fn extract_market_price(json: &Value) -> Result<f64, Box<dyn std::error::Error>> {
    extract_field(&json, |t| t.as_f64())
}

fn extract_field<T>(json: &Value, extractor: fn(&Value) -> Option<T>) -> Result<T, Box<dyn std::error::Error>> {
    let selector = Selector::new(JSON_PATH_MARKET_RAW_PRICE)?;

    selector.find(json)
        .flat_map(extractor)
        .take(1)
        .last()
        .ok_or(Box::new(Error::new(ErrorKind::Other, "Field not found")))
}

impl QuoteRetriever for QuoteService {
    fn find(&self, symbol: &str) -> Option<Quote> {
        match self.quotes.read().unwrap().get(symbol) {
            Some(v) => Some(v.clone()),
            _ => None
        }
    }

    fn available_quotes(&self) -> Vec<String> {
        self.quotes.read()
            .unwrap()
            .keys()
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use crate::financial::extract_market_price;

    #[test]
    fn json_path() {

        let jsondoc= r#"{"quoteSummary":{"error":null,"result":[{"price":{"averageDailyVolume10Day":{},"averageDailyVolume3Month":{},"circulatingSupply":{},"currency":"USD","currencySymbol":"$","exchange":"NMS","exchangeDataDelayedBy":0,"exchangeName":"NasdaqGS","fromCurrency":null,"lastMarket":null,"longName":"Microsoft Corporation","marketCap":{"fmt":"2.30T","longFmt":"2,295,241,637,888.00","raw":2295241637888},"marketState":"REGULAR","maxAge":1,"openInterest":{},"postMarketChange":{},"postMarketPrice":{},"preMarketChange":{"fmt":"5.89","raw":5.89001},"preMarketChangePercent":{"fmt":"1.99%","raw":0.0198511},"preMarketPrice":{"fmt":"302.60","raw":302.6},"preMarketSource":"FREE_REALTIME","preMarketTime":1643293799,"priceHint":{"fmt":"2","longFmt":"2","raw":2},"quoteSourceName":"Nasdaq Real Time Price","quoteType":"EQUITY","regularMarketChange":{"fmt":"9.45","raw":9.450012},"regularMarketChangePercent":{"fmt":"3.18%","raw":0.03184932},"regularMarketDayHigh":{"fmt":"307.16","raw":307.16},"regularMarketDayLow":{"fmt":"301.35","raw":301.35},"regularMarketOpen":{"fmt":"302.66","raw":302.66},"regularMarketPreviousClose":{"fmt":"296.71","raw":296.71},"regularMarketPrice":{"fmt":"306.16","raw":306.16},"regularMarketSource":"FREE_REALTIME","regularMarketTime":1643300103,"regularMarketVolume":{"fmt":"19.91M","longFmt":"19,909,759.00","raw":19909759},"shortName":"Microsoft Corporation","strikePrice":{},"symbol":"MSFT","toCurrency":null,"underlyingSymbol":null,"volume24Hr":{},"volumeAllCurrencies":{}}}]}}"#;

        // Parse JSON document
        let json: Value = serde_json::from_str(jsondoc).unwrap();

        let price = extract_market_price(&json).expect("Value needs to be extracted");

        assert_eq!(price, 306.16);
    }
}