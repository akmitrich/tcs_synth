use base64::Engine;
use tokio::io::AsyncWriteExt;
use tokio_stream::StreamExt;

mod tcs {
    tonic::include_proto!("tinkoff.cloud.tts.v1");
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let sample_rate_hertz = 8000;
    let api_key = std::env::var("TCS_APIKEY").unwrap();
    let secret_key = base64::engine::general_purpose::STANDARD
        .decode(std::env::var("TCS_SECRET").unwrap())
        .unwrap();
    let mut args = std::env::args();
    let output_path = args.nth(1).unwrap();
    let text = args.next().unwrap_or_else(|| {
        String::from("Оптимал Сити Текнолоджис предоставляет наилучший сервис.")
    });
    let voice = args.next().unwrap_or_else(|| String::from("flirt"));

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
    let mut req = tonic::Request::new(tcs::SynthesizeSpeechRequest {
        input: Some(tcs::SynthesisInput {
            text,
            ssml: String::new(),
        }),
        voice: Some(tcs::VoiceSelectionParams { name: voice }),
        audio_config: Some(tcs::AudioConfig {
            audio_encoding: tcs::AudioEncoding::Linear16.into(),
            speaking_rate: 1.0,
            pitch: 1.0,
            sample_rate_hertz,
        }),
    });
    req.metadata_mut()
        .append("authorization", format!("Bearer {}", jwt).parse().unwrap());
    let resp = client.streaming_synthesize(req).await.unwrap();
    println!("Request ID {:?}", resp.metadata().get("x-request-id"));
    let mut chunks = resp.into_inner();
    let mut output = tokio::fs::File::create(output_path).await.unwrap();
    while let Some(chunk) = chunks.next().await {
        match chunk {
            Ok(data) => {
                println!("data: {} bytes.", data.audio_chunk.len());
                output.write_all(data.audio_chunk.as_slice()).await.unwrap();
            }
            Err(e) => println!("FAILED: {:?}", e),
        }
    }
    println!("OK.");
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
