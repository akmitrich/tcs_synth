fn main() {
    println!("cargo::rerun-if-changed=proto/tts.proto");
    tonic_build::configure()
        .compile(&["proto/tts.proto"], &[".", "googleapis"])
        .unwrap();
}
