//! Ed25519 doğrulayıcı yükleme.
//!
//! Python `cryptography`'ın `verify`'ı RFC 8032 **strict** doğrular; burada da
//! `verify_strict` kullanılır — iki yol aynı biletleri kabul etmeli.

use ed25519_dalek::VerifyingKey;

use crate::b64;
use crate::Hata;

/// base64 (standart veya url-safe, paddingli veya değil) → Ed25519 açık anahtar.
pub fn dogrulayici(acik_anahtar_b64: &str) -> Result<VerifyingKey, Hata> {
    let ham = b64::coz(acik_anahtar_b64)?;
    let dizi: &[u8; 32] = ham.as_slice().try_into().map_err(|_| Hata("acik-anahtar-32-bayt-degil"))?;
    VerifyingKey::from_bytes(dizi).map_err(|_| Hata("acik-anahtar-ed25519-degil"))
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::sabitler;

    #[test]
    // Gömülü açık anahtarların dördü de gerçekten geçerli Ed25519 noktası olmalı.
    // Bu test yazım (transkripsiyon) hatalarını yakalar — anahtarı yanlış
    // kopyalanan bir modül hiçbir müşterinin biletini doğrulayamaz.
    fn gomulu_acik_anahtarlar_gecerli() {
        for anahtar in [
            sabitler::KDV_SATIS_ACIK_ANAHTARI,
            sabitler::PAKET_YETKI_ACIK_ANAHTARI,
            sabitler::EKESINTI_ACIK_ANAHTARI,
            sabitler::LMZ_ACIK_ANAHTARI,
        ] {
            assert!(
                dogrulayici(anahtar).is_ok(),
                "gömülü açık anahtar çözülemedi: {anahtar}"
            );
        }
    }
}
