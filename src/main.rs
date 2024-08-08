use base64::Engine;
use serde_json::{json, Value};
use std::io::Write;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let sample_rate = 8000;
    let api_key = std::env::var("TCS_APIKEY").unwrap();
    let secret_key = base64::engine::general_purpose::STANDARD
        .decode(std::env::var("TCS_SECRET").unwrap())
        .unwrap();
    println!("Run with {:?} -- {:?}", api_key, secret_key);
    let mut args = std::env::args();
    let _ = args.next();
    let text = args.next().unwrap();
    let output_path = args.next().unwrap();

    let jwt = generate(&api_key, &secret_key);
    let req_body = json!({"input": {"text": text}, "audioConfig": {"audioEncoding": "LINEAR16","sampleRateHertz": sample_rate}, "voice": {"name": "alyona"}});
    let client = reqwest::Client::new();
    let req = client
        .post("https://api.tinkoff.ai:443/v1/tts:synthesize")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", jwt))
        .json(&req_body);
    let resp = req.send().await.unwrap();
    println!("Response from TTS: {:?}", resp);
    let status_code = resp.status();
    let resp_json = resp.json::<Value>().await.unwrap();
    println!("TTS status code {:?}", status_code);

    let wav = base64::engine::general_purpose::STANDARD
        .decode(resp_json["audio_content"].as_str().unwrap())
        .unwrap();
    let mut f = std::fs::File::create(output_path).unwrap();
    f.write_all(wav.as_slice()).unwrap();
}

fn generate(api_key: &str, secret_key: &[u8]) -> String {
    let claims = json!({
        "iss": "synth",
        "sub": "akmitrich",
        "aud": "tinkoff.cloud.tts",
        "exp": chrono::Local::now().timestamp() + 3600,
    });
    let header = jsonwebtoken::Header {
        kid: Some(api_key.to_owned()),
        alg: jsonwebtoken::Algorithm::HS256,
        ..Default::default()
    };
    jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret_key),
    )
    .unwrap()
}
