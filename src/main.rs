use dotenv::dotenv;
use std::env;
use std::{thread, time};

use teloxide::prelude::*;

use reqwest;
use scraper;

#[derive(Debug, Clone, PartialEq)]
struct BinanceInfo {
    block: i128,
    date: String,
    sum: String,
    balance: String,
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let ten_seconds = time::Duration::from_secs(5);
    let mut last_operation: BinanceInfo = BinanceInfo {
        block: 0,
        date: "".to_string(),
        sum: "".to_string(),
        balance: "".to_string(),
    };

    dotenv().ok();

    teloxide::enable_logging!();
    log::info!("Starting BinanceBTCBot...");

    let bot = Bot::from_env().auto_send();

    loop {
        let possible_new_operation: BinanceInfo = new_operation().await;
        if last_operation != possible_new_operation {
            last_operation = possible_new_operation;

            let telegram_text = format!(
                "На сайте появилась новая транзакция!\nБлок: {}\nДата: {}\nСумма: {}\nТекущий баланс: {} BTC",
                last_operation.block, last_operation.date, last_operation.sum, last_operation.balance
            );
            bot.send_message(558612972, telegram_text)
                .send()
                .await
                .unwrap();
        }

        thread::sleep(ten_seconds);
    }
}

async fn new_operation() -> BinanceInfo {
    let responce = reqwest::get(
        "https://bitinfocharts.com/ru/bitcoin/address/34xp4vRoCGJym3xR7yCVPFHoCNxv4Twseo",
    )
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    let fragment = scraper::Html::parse_fragment(responce.as_str());
    let tr_selector = scraper::Selector::parse("tr.trb").unwrap();

    let tr = fragment.select(&tr_selector).next().unwrap();
    let text = tr.text().collect::<Vec<_>>();

    BinanceInfo {
        block: text[0].parse::<i128>().unwrap(),
        date: text[1].to_string(),
        sum: text[3].to_string(),
        balance: text[5].to_string(),
    }
}
