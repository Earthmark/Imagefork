use base64::Engine;
use rand::RngCore;

pub fn generate_token() -> String {
    let mut token = [0; 32];
    rand::thread_rng().try_fill_bytes(&mut token).unwrap();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(token)
}
