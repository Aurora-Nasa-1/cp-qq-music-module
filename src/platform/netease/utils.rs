use aes::Aes128;
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockEncryptMut, KeyInit};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;

type Aes128EcbEnc = ecb::Encryptor<Aes128>;

const EAPI_KEY: &[u8; 16] = b"e82ckenh8dichen8";
const ALBUM_CACHE_KEY: &[u8; 16] = b")(13daqP@ssw0rd~";
const MAGIC: &str = "36cd479b6b5";

fn aes_ecb_pkcs7_encrypt(plaintext: &[u8], key: &[u8; 16]) -> Vec<u8> {
    Aes128EcbEnc::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}

pub(super) fn album_cache_key(id: &str) -> String {
    // 专辑详情接口要求 cache_key 为固定格式串经过 AES-ECB 后的 base64。
    let plain = format!("e_r=false&id={id}");
    BASE64.encode(aes_ecb_pkcs7_encrypt(plain.as_bytes(), ALBUM_CACHE_KEY))
}

pub(super) fn eapi_params(api_path: &str, body_json: &str) -> String {
    // EAPI params: 先按官方拼接规则计算 md5，再把明文做 AES-ECB 并输出大写 HEX。
    let md5_src = format!("nobody{api_path}use{body_json}md5forencrypt");
    let digest = format!("{:x}", md5::compute(md5_src.as_bytes()));
    let plain = format!("{api_path}-{MAGIC}-{body_json}-{MAGIC}-{digest}");
    hex::encode_upper(aes_ecb_pkcs7_encrypt(plain.as_bytes(), EAPI_KEY))
}
