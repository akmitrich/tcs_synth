use base64::Engine;

mod tcs {
    tonic::include_proto!("tinkoff.cloud.tts.v1");
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let sample_rate = 8000;
    let api_key = std::env::var("TCS_APIKEY").unwrap();
    let secret_key = base64::engine::general_purpose::STANDARD
        .decode(std::env::var("TCS_SECRET").unwrap())
        .unwrap();

    let jwt = generate(&api_key, &secret_key);
    println!("JWT={}", jwt);
    let mut client = grpc_client().await;
    println!("CLIENT: {:?}", client);
    let mut voice_req = tonic::Request::new(tcs::ListVoicesRequest {});
    voice_req
        .metadata_mut()
        .append("authorization", format!("Bearer {}", jwt).parse().unwrap());
    let voice_resp = client.list_voices(voice_req).await.unwrap();
    println!("VOICES: {:?}", voice_resp.into_inner().voices);
}

fn generate(api_key: &str, secret_key: &[u8]) -> String {
    let claims = serde_json::json!({
        "iss": "synth",
        "sub": "akmitrich",
        "aud": "tinkoff.cloud.tts",
        "exp": chrono::Local::now().timestamp() + 60,
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

fn tls_config() -> tonic::transport::ClientTlsConfig {
    tonic::transport::ClientTlsConfig::new().with_native_roots()
}

async fn grpc_client() -> tcs::text_to_speech_client::TextToSpeechClient<tonic::transport::Channel>
{
    let channel = tonic::transport::Channel::from_static("https://api.tinkoff.ai:443")
        .tls_config(tls_config())
        .unwrap()
        .connect()
        .await
        .unwrap();
    tcs::text_to_speech_client::TextToSpeechClient::new(channel)
}
