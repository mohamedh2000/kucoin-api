use::std::time::SystemTime;
use::crypto::hmac::Hmac;
use::crypto::sha2::Sha256;
use::crypto::mac::Mac;
use::base64::{ encode };
use::reqwest;
use::serde_json::Value;
use std::panic::{set_hook};
use std::{collections::HashMap};

#[derive()]
struct KucoinCred<'a> { 
    kucoin_api_key : &'a str,
    kucoin_passphrase : &'a str,
    kucoin_secret : &'a str,
    kucoin_base_uri : &'a str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    set_hook(Box::new(|_| {
        ();
    }));

    let kucoin_acc_endpoint : &str =  "/api/v1/accounts";
    let kucoin_price_endpoint : &str = "/api/v1/market/orderbook/level1";

    let current_user :KucoinCred = KucoinCred {
        kucoin_base_uri : "https://api.kucoin.com",
        kucoin_secret : "b96b57e7-8534-4aa0-9a44-61e1666c5e67",
        kucoin_api_key : "61c83393d98e2200016519ea",
        kucoin_passphrase : "M?m5cBP?ko4fJG9A", 
    };
    
    let res_str: String = get_kucoin_response(&kucoin_acc_endpoint, &current_user).await?;

    let new_res = serde_json::from_str::<HashMap<String, Value>>(&res_str).unwrap();

    let acc_balance: &Vec<Value> = new_res.get("data").unwrap().as_array().unwrap();

    for n in 0..acc_balance.len() {

        let current_obj = &acc_balance[n];
        let sym : &str = &current_obj["currency"].as_str().unwrap();
        let balance : &f64 = &current_obj["balance"].as_str().unwrap().parse::<f64>().unwrap();

        let price_endpoint : String = format!("{}?symbol={}-USDT", &kucoin_price_endpoint,sym);
       
        let price = get_kucoin_response(&price_endpoint, &current_user).await?;

        let price_res = serde_json::from_str::<HashMap<String, Value>>(&price).unwrap();

        let ticker_price : Result<f64,()> = Ok(price_res.get("data").unwrap()["price"].as_str().unwrap().parse::<f64>().unwrap());

        if *balance != 0.0 && ticker_price.is_ok() { println!("{:.2} {} ({:.2} USD)", balance, sym, ticker_price.unwrap() * balance)};
    }

    Ok(())
}

/**
 * Use API-Secret to encrypt the prehash string {timestamp+method+endpoint+body} with sha256 HMAC. 
 * The request body is a JSON string and need to be the same with the parameters passed by the API. 
 * After that, use base64-encode to encrypt the result in step 1 again.
 * */
fn sign(str_input : &str, secret_key : &str) -> String {
    //need to encrypt the secret possibly
    let mut temp = Hmac::new(Sha256::new(), secret_key.as_bytes());
    temp.input(str_input.as_bytes());
    let temp_res = temp.result();
    let result_str = encode(temp_res.code());
    return result_str;
}

fn get_current_time_stamp() -> u128 {
    return SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
}

async fn get_kucoin_response(kucoin_endpoint : &str, current_user : &KucoinCred<'_>) -> Result<String, reqwest::Error> {
    
    let uri = format!("{}{}",current_user.kucoin_base_uri, kucoin_endpoint);

    let client = reqwest::Client::new();
 
    //constructing the timestamp 
    let time_stamp : u128 = get_current_time_stamp();

    let sign_str : &str = &format!("{}{}{}{}", &time_stamp, "GET", kucoin_endpoint, "");
    let kc_sign : String = sign(sign_str, &current_user.kucoin_secret);

    let kc_pass_sign : String = sign(&current_user.kucoin_passphrase, &current_user.kucoin_secret);
    let kucoin_version : &str = "2";

    let res = client.get(uri)
        .header("KC-API-KEY", current_user.kucoin_api_key)  
        .header("KC-API-SIGN", kc_sign)
        .header("KC-API-TIMESTAMP", time_stamp.to_string())
        .header("KC-API-PASSPHRASE", kc_pass_sign)
        .header("KC-API-KEY-VERSION", kucoin_version)
        .header("content-type", "application/json")
        .body("").send().await?.text().await?;
    return Ok(res);
}