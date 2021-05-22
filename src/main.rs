use dotenv::dotenv;
use std::env;
use std::{thread, time};

use teloxide::prelude::*;

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
    let ten_seconds = time::Duration::from_secs(10);
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
        let possible_new_operation: BinanceInfo = match get_new_operation().await {
            Ok(new_operation) => new_operation,
            Err(err) => {
                log::info!("Failed to get new operation due to: {}", err);
                continue;
            }
        };

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

async fn get_new_operation() -> Result<BinanceInfo, Box<dyn std::error::Error>> {
    let responce = reqwest::get(
        "https://bitinfocharts.com/ru/bitcoin/address/34xp4vRoCGJym3xR7yCVPFHoCNxv4Twseo",
    )
    .await;

    let responce = responce?;

    if responce.status().is_success() {
        let text = responce.text().await?;

        let fragment = scraper::Html::parse_fragment(text.as_str());
        let tr_selector = scraper::Selector::parse("tr.trb").unwrap();

        let tr = fragment.select(&tr_selector).next().unwrap();
        let text = tr.text().collect::<Vec<_>>();

        return Ok(BinanceInfo {
            block: text[0].parse::<i128>().unwrap(),
            date: text[1].to_string(),
            sum: text[3].to_string(),
            balance: text[5].to_string(),
        });
    }
    if responce.status().is_client_error() {
        return Err("BAD REQUEST".into());
    }

    Err("Something Went Wrong...".into())
}
