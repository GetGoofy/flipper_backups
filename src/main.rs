mod helpers;
mod tables;
mod types;

use chrono::Local;
use clokwerk::{Job, Scheduler, TimeUnits};
use postgres::{Client as PostgresClient, NoTls};
use reqwest::blocking::Client;

use dotenvy;
use std::time::Duration;
use std::{env, thread};

use crate::tables::prepare_table_tsm_pricing_data_weekly;

fn main() {
    dotenvy::dotenv().ok();

    let mut scheduler = Scheduler::new();

    scheduler.every(7.day()).at("10:00 am").run( || {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let connect_to_db = PostgresClient::connect(&database_url, NoTls);

        let mut db_client = connect_to_db.expect("An error happened when attempting to connect to the DB");

        prepare_table_tsm_pricing_data_weekly(&mut db_client);

        let http_client = Client::new();

        let tsm_auth_body: types::TsmAuthBody = types::TsmAuthBody {
            client_id: "c260f00d-1071-409a-992f-dda2e5498536".to_string(),
            grant_type: "api_token".to_string(),
            scope: "app:realm-api app:pricing-api".to_string(),
            token: helpers::use_variable_TSM_API_KEY(),
        };

        let resp_tsm_access_token = http_client
            .post("https://auth.tradeskillmaster.com/oauth2/token")
            .json(&tsm_auth_body)
            .send();

        let resp_tsm_access_token = match resp_tsm_access_token {
            Ok(r) => r,
            Err(e) => {
                println!("HTTP request failed: {}", e);
                return;
            }
        };

        let resp_tsm_access_token: types::TsmAuthResponse = match resp_tsm_access_token.json() {
            Ok(data) => data,
            Err(e) => {
                println!("JSON deserialization failed: {}", e);
                return;
            }
        };

        let resp_tsm_region_pricing_data = http_client
            .get("https://pricing-api.tradeskillmaster.com/region/1")
            .bearer_auth(&resp_tsm_access_token.access_token)
            .send();

        let resp_tsm_region_pricing_data = match resp_tsm_region_pricing_data {
            Ok(r) => r,
            Err(e) => {
                println!("HTTP request failed: {}", e);
                return;
            }
        };

        let mut resp_tsm_region_pricing_data: Vec<types::TsmPricingDataResponse> =
            match resp_tsm_region_pricing_data.json::<Vec<types::TsmPricingDataResponse>>() {
                Ok(data) => data.into_iter().filter(|item| item.item_id.is_some())
            .collect(),
                Err(e) => {
                    println!("JSON deserialization failed: {}", e);
                    return;
                }
            };

        resp_tsm_region_pricing_data.sort_by_key(|item| item.item_id.unwrap() as u64);

        let date_string = Local::now().format("%d/%m/%Y").to_string();

        let connect_to_db = PostgresClient::connect(&database_url, NoTls);

        let mut db_client = connect_to_db.expect("An error happened when attempting to connect to the DB");

        let check_if_date_already_exists_in_db = &db_client.query(
    "SELECT 1 FROM tsm_pricing_data_weekly WHERE created_at = $1 LIMIT 1",
    &[&date_string]
        );

        let check_if_date_already_exists_in_db = match check_if_date_already_exists_in_db {
            Ok(x) => x,
            Err(e) => {
                println!("Query to DB has failed: {e}");
                return;
            }
        };

        if check_if_date_already_exists_in_db.len() > 0 {
            println!("There is already TSM pricing data available in the DB using that date, skipping.");
            return;
        }

        
        println!("Starting transaction...");

        let mut transaction = db_client.transaction().unwrap();

        for item in resp_tsm_region_pricing_data {
            println!("{:#?}", item);
            let execute = transaction.execute(
                "INSERT INTO tsm_pricing_data_weekly (region_id, item_id, avg_sale_price, sold_per_day, sale_rate, quantity, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)"
                , &[&item.region_id, &item.item_id.unwrap(), &item.avg_sale_price, &item.sold_per_day, &item.sale_rate, &item.quantity, &date_string]);
            
            match execute {
                Ok(_) => {},
                Err(e) => {
                    println!("Transaction operation has failed: {e}");
                    return;
            }
        }
        }
        
        match transaction.commit() {
            Ok(tx) => tx,
            Err(e) => {
                println!("Transaction to DB has failed: {e}");
                return;
            }        
        };

        println!("TSM pricing data saved successfully to the DB");

    });

    // Run the scheduler in a loop, checking for pending tasks
    loop {
        scheduler.run_pending();
        thread::sleep(Duration::from_millis(100)); // Check every 100ms
    }
}