use std::env;

use anyhow::{anyhow, Context};
use dotenv::dotenv;
use teloxide::requests::{Request, Requester, RequesterExt};

#[derive(Debug, Clone, PartialEq)]
struct BinanceInfo {
    block_height: i128,
    block_creation_date: String,
    transfer_amount: String,
    balance: String,
}

#[tokio::main]
async fn main() {
    run().await;
}

async fn run() {
    let operations_polling_interval = std::time::Duration::from_secs(10);
    let delay_between_failing_attempts = std::time::Duration::from_secs(3);

    let mut last_reported_operation: Option<BinanceInfo> = None;

    dotenv().ok();

    teloxide::enable_logging!();
    log::info!("Starting BinanceBTCBot...");

    let bot = teloxide::Bot::from_env().auto_send();

    loop {
        tokio::time::sleep(operations_polling_interval).await;

        let latest_operation: BinanceInfo = match get_latest_operation().await {
            Ok(operation) => operation,
            Err(err) => {
                log::info!(
                    "Failed to get latest operation due to: {}. Retrying in {} seconds...",
                    err,
                    operations_polling_interval.as_secs_f64()
                );
                continue;
            }
        };

        match &last_reported_operation {
            None => {
                last_reported_operation = Some(latest_operation);
                continue;
            }
            Some(last_reported_operation) if last_reported_operation == &latest_operation => {
                continue;
            }
            Some(_) => {}
        }

        let telegram_text = format!(
            "На сайте появилась новая транзакция!\nБлок: {}\nДата: {}\nСумма: {}\nТекущий баланс: {} BTC",
            latest_operation.block_height,
            latest_operation.block_creation_date,
            latest_operation.transfer_amount,
            latest_operation.balance
        );
        while let Err(err) = bot.send_message(558612972, &telegram_text).send().await {
            log::info!(
                "Failed to sent Telegram notification due to: {}. Retrying in {} seconds...",
                err,
                delay_between_failing_attempts.as_secs_f64()
            );
            tokio::time::sleep(delay_between_failing_attempts).await;
        }

        last_reported_operation = Some(latest_operation);
    }
}

async fn get_latest_operation() -> Result<BinanceInfo, anyhow::Error> {
    let response = reqwest::get(
        "https://bitinfocharts.com/ru/bitcoin/address/34xp4vRoCGJym3xR7yCVPFHoCNxv4Twseo",
    )
    .await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "fetching bitinfocharts.com failed with HTTP code: {}",
            response.status()
        ));
    }

    let fragment = scraper::Html::parse_fragment(response.text().await?.as_str());

    let cells_selector =
        scraper::Selector::parse("tr.trb td").expect("failed to parse the cells selector");
    let mut cells = fragment.select(&cells_selector);
    let block_height = cells
        .next()
        .context("failed to find block height cell")?
        .text()
        .next()
        .context("failed to find block height")?
        .parse()
        .context("failed to parse block height")?;
    let block_creation_date = cells
        .next()
        .context("failed to find block creation date cell")?
        .text()
        .next()
        .context("failed to find block creation date")?
        .to_string();
    let transfer_amount = cells
        .next()
        .context("failed to find transfer amount cell")?
        .text()
        .next()
        .context("failed to find transfer amount")?
        .to_string();
    let balance = cells
        .next()
        .context("failed to find balance cell")?
        .text()
        .next()
        .context("failed to find balance")?
        .to_string();

    Ok(BinanceInfo {
        block_height,
        block_creation_date,
        transfer_amount,
        balance,
    })
}
