use::std::time::SystemTime;
use::crypto::hmac::Hmac;
use::crypto::sha2::Sha256;
use::crypto::mac::Mac;
use::base64::{ encode };
use::reqwest;
use::serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    const KUCOIN_API_KEY : &str = "61c83393d98e2200016519ea";
    const KUCOIN_PASSPHRASE : &str = "M?m5cBP?ko4fJG9A";
    const KUCOIN_SECRET : &str = "b96b57e7-8534-4aa0-9a44-61e1666c5e67";
    const KUCOIN_BASE_URI : &str = "https://api.kucoin.com";
    const KUCOIN_ENDPOINT : &str = "/api/v1/accounts";

    //constructing the timestamp 
    let time_stamp : u128 = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();

    let sign_str : &str = &format!("{}{}{}{}", &time_stamp, "GET", &KUCOIN_ENDPOINT, "");
    let kc_sign : String = sign(sign_str, &KUCOIN_SECRET);

    let kc_pass_sign : String = sign(KUCOIN_PASSPHRASE, &KUCOIN_SECRET);
    let kucoin_version : &str = "2";

    let uri = format!("{}{}",KUCOIN_BASE_URI, KUCOIN_ENDPOINT);

    let client = reqwest::Client::new();

    let res = client.get(uri)
            .header("KC-API-KEY", KUCOIN_API_KEY)  
            .header("KC-API-SIGN", kc_sign)
            .header("KC-API-TIMESTAMP", time_stamp.to_string())
            .header("KC-API-PASSPHRASE", kc_pass_sign)
            .header("KC-API-KEY-VERSION", kucoin_version)
            .header("content-type", "application/json")
            .body("")
            .send()
            .await?.text().await?;

    let new_res = serde_json::from_str::<HashMap<String, Value>>(&res).unwrap();

    let acc_balance: &Vec<Value> = new_res.get("data").unwrap().as_array().unwrap();

    for n in 0..=acc_balance.len() - 1 {
        //Object({"available": String("502.54391194"), "balance": String("502.54391194"), "currency": String("KDA"), "holds": String("0"), "id": String("6182514dc42972000142f847"), "type": String("trade")})
        let current_obj = &acc_balance[n];
        let sym : &str = &current_obj["currency"].as_str().unwrap();
        let balance : &f64 = &current_obj["balance"].as_str().unwrap().parse::<f64>().unwrap();
        if *balance != 0.0 { println!("{} : {}", sym, balance)}
    }

    //let mut file : File = File::create("./resFile")?;

    //file.write_all(new_res.unwrap().get("data").unwrap().to_string().as_bytes());
    
    Ok(())
}


/**
 * For the header of KC-API-SIGN: Use API-Secret to encrypt the prehash string {timestamp+method+endpoint+body} 
 * with sha256 HMAC. The request body is a JSON string and need to be the same with the parameters passed by the API. 
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