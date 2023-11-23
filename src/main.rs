use std::io::Write;

use anyhow::Context;
use base64::Engine;
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_rate = 8000;
    let api_key = "9L1tOnU1okWGRrO2oB2RZTOCSoZEMVofCjhB3lYREgw=";
    let secret_key = base64::engine::general_purpose::STANDARD
        .decode("Dm8zMWsTFFBP3MwMvHKkNM1L+nmvSInWqMX/sdAWoqo=")
        .unwrap();
    let mut args = std::env::args();
    let _ = args.next();
    let text = args
        .next()
        .ok_or_else(|| anyhow::Error::msg("First argument is the text to synthesize"))?;
    let output_path = args
        .next()
        .ok_or_else(|| anyhow::Error::msg("Second argument is the path to save PCM-stream"))?;

    let jwt = generate(api_key, &secret_key);
    let req_body = json!({"input": {"text": text}, "audioConfig": {"audioEncoding": "LINEAR16","sampleRateHertz": sample_rate}, "voice": {"name": "alyona"}});
    let client = reqwest::Client::new();
    let req = client
        .post("https://api.tinkoff.ai:443/v1/tts:synthesize")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}", jwt?))
        .json(&req_body);
    let resp = req.send().await?;
    println!("Response from TTS: {:?}", resp);
    let status_code = resp.status();
    let resp_json = resp.json::<Value>().await?;
    println!("TTS status code {:?}", status_code);

    let wav = base64::engine::general_purpose::STANDARD
        .decode(
            resp_json["audio_content"]
                .as_str()
                .ok_or_else(|| anyhow::Error::msg("audio content not found in TTS response"))?,
        )
        .unwrap();
    let mut f = std::fs::File::create(output_path)?;
    f.write_all(wav.as_slice())?;
    Ok(())
}

fn generate(api_key: &str, secret_key: &[u8]) -> anyhow::Result<String> {
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
    .context("jwt encode fails")
}
