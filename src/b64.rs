//! Base64 esnekliği.
//!
//! Python ve Worker tarafı karışık alfabe kullanıyor: bilet bölümleri
//! url-safe-sans-padding (`worker/index.js` `_b64url`), gömülü açık anahtarlar
//! ise standart alfabe + padding olabiliyor (`lisans.py:320`'deki
//! `EvR...+RIgW91J6DLVSclZyJfc=`). `_b64url_coz` (`lisans.py:369-372`) her
//! ikisini de sindirir; burası aynı hoşgörüyı uygular.

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD_NO_PAD, URL_SAFE_NO_PAD};

use crate::Hata;

/// Her iki alfabeyi ve paddingli/padding'siz biçimi kabul eder.
///
/// Kural (Python'un `-`→`+`, `_`→`/` çevriminin birebir karşılığı): metinde
/// `-` veya `_` varsa url-safe, yoksa standart alfabe ile çözülür; sonda
/// `=` atıkları yok sayılır.
pub fn coz(metin: &str) -> Result<Vec<u8>, Hata> {
    let temiz = metin.trim().trim_end_matches('=');
    let ham = if temiz.contains(['-', '_']) {
        URL_SAFE_NO_PAD.decode(temiz.as_bytes())
    } else {
        STANDARD_NO_PAD.decode(temiz.as_bytes())
    };
    ham.map_err(|_| Hata("base64-cozulemedi"))
}

/// Worker'ın `_b64url`'ı ile aynı çıktı: url-safe, padding'siz.
pub fn kodla_urlsafe(veri: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(veri)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn uc_akista_da_ayni() {
        // "ü" = 0xC3 0xFC → standart "w/w=" (paddingli) ↔ url-safe "w_w"
        // (padding'siz). Python `_b64url_coz` ikisini de sindirir; burası da.
        assert_eq!(coz("w/w=").unwrap(), vec![0xC3, 0xFC]);
        assert_eq!(coz("w_w").unwrap(), vec![0xC3, 0xFC]);
        // [0xFB,0xFF]: gömülü Anahtar anahtarları standart, worker biletleri
        // url-safe alfabe kullanır — ikisi de beslenebilmeli.
        assert_eq!(coz("+/8=").unwrap(), vec![0xFB, 0xFF]);
        assert_eq!(coz("-_8").unwrap(), vec![0xFB, 0xFF]);
    }

    #[test]
    fn bozuk_girdi_hata() {
        assert_eq!(coz("!!!"), Err(Hata("base64-cozulemedi")));
        assert_eq!(coz("ab"), Err(Hata("base64-cozulemedi"))); // artık bitleri boş
    }

    #[test]
    fn kodlama_worker_ile_ayni() {
        // worker _b64url: url-safe + padding yok.
        assert_eq!(kodla_urlsafe(&[0xFB, 0xFF, 0xFE]), "-__-");
    }
}
