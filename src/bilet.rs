//! Sunucu bileti: `b64url(govde_json).b64url(Ed25519_imza)`.
//!
//! Kaynak (birebir doğrulandı):
//! - Üretim: `worker/index.js:798-818` — imzalanan yük **govde JSON'unun
//!   ham UTF-8 baytlarıdır**, b64 metni DEĞİL.
//! - Tüketim: `lisans.py:375-392` (`_bilet_dogrula`) — 2 bölüm, imzasız
//!   bilet None; gövde dict değilse None. Fail-closed.
//!
//! Aynı biçim yetki bileti ve modül bileti için kullanılır (`modul` alanı
//! ayırır); E-Kesinti lease token'ı da aynı şekildedir ama canonical-JSON
//! farkı ve farklı gövde şeması nedeniyle BUGÜNLÜK kapsamlarımızda değildir.

use ed25519_dalek::Signature;

use crate::Hata;
use crate::b64;
use crate::imza::dogrulayici;

/// Çözümlenmiş bilet gövdesi. Python `.get()` hoşgörüsüyle: alan yoksa
/// varsayılan (`hwid`/`until` boş string, `iat_ms`/`modul` None).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Govde {
    pub hwid: String,
    pub until: String,
    pub iat_ms: Option<i64>,
    /// Yoksa None: ESKİ bilet. Çağıran `karar::bilet_karari` bunu "kdv" sayar.
    pub modul: Option<String>,
}

/// İmzalı bileti doğrular ve gövdesini çözümler.
///
/// Başarısızlık (biçim/imza/JSON) `Err`'dir ve ÇAĞIRAN için anlamı
/// "bilet yoktur": yerel kaydı silmeden denemeye düş.
pub fn bilet_dogrula(bilet: &str, acik_anahtar_b64: &str) -> Result<Govde, Hata> {
    let parcalar: Vec<&str> = bilet.trim().split('.').collect();
    if parcalar.len() != 2 {
        return Err(Hata("bilet-iki-parca-olmali"));
    }
    let govde_bayt = b64::coz(parcalar[0])?;
    let imza_bayt = b64::coz(parcalar[1])?;
    let imza = Signature::from_slice(&imza_bayt).map_err(|_| Hata("imza-64-bayt-degil"))?;
    dogrulayici(acik_anahtar_b64)?
        .verify_strict(&govde_bayt, &imza)
        .map_err(|_| Hata("imza-dogrulanamadi"))?;

    let veri: serde_json::Value =
        serde_json::from_slice(&govde_bayt).map_err(|_| Hata("govde-json-degil"))?;
    let nesne = veri.as_object().ok_or(Hata("govde-nesne-degil"))?;
    let string_al = |ad: &str| -> String {
        nesne.get(ad).and_then(|v| v.as_str()).unwrap_or("").to_string()
    };
    Ok(Govde {
        hwid: string_al("hwid"),
        until: string_al("until"),
        iat_ms: nesne.get("iat").and_then(serde_json::Value::as_i64),
        modul: nesne.get("modul").and_then(|v| v.as_str()).map(str::to_string),
    })
}

#[cfg(test)]
mod testler {
    use super::*;
    use ed25519_dalek::Signer;

    /// Deterministik test anahtarı (gizli anahtar DEĞİL, yalnız test).
    fn test_anahtarlari() -> (ed25519_dalek::SigningKey, String) {
        let imzalayan = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
        let acik = b64::kodla_urlsafe(&imzalayan.verifying_key().to_bytes());
        (imzalayan, acik)
    }

    fn bilet_uret(imzalayan: &ed25519_dalek::SigningKey, govde_json: &str) -> String {
        let govde = govde_json.as_bytes();
        let imza = imzalayan.sign(govde); // worker ile aynı: ham govde baytları
        format!("{}.{}", b64::kodla_urlsafe(govde), b64::kodla_urlsafe(&imza.to_bytes()))
    }

    #[test]
    fn canli_worker_bicimi_dogrulanir() {
        let (imzalayan, acik) = test_anahtarlari();
        let govde_json = r#"{"hwid":"KDV-ABCD-EFGH-JKMN-PQRS","until":"2027-01-15","iat":1763000000000,"modul":"mizan"}"#;
        let bilet = bilet_uret(&imzalayan, govde_json);
        let govde = bilet_dogrula(&bilet, &acik).unwrap();
        assert_eq!(govde.hwid, "KDV-ABCD-EFGH-JKMN-PQRS");
        assert_eq!(govde.until, "2027-01-15");
        assert_eq!(govde.iat_ms, Some(1763000000000));
        assert_eq!(govde.modul.as_deref(), Some("mizan"));
    }

    #[test]
    fn tahrif_edilmis_govde_dusur() {
        let (imzalayan, acik) = test_anahtarlari();
        let bilet =
            bilet_uret(&imzalayan, r#"{"hwid":"X","until":"2027-01-15","iat":1,"modul":"kdv"}"#);
        // until'i 2099'a çekmeye çalışan saldırgan:
        let (govde_b64, imza_b64) = bilet.split_once('.').unwrap();
        let ham = b64::coz(govde_b64).unwrap();
        let tahrif = String::from_utf8(ham).unwrap().replace("2027", "2099");
        let sahte = format!("{}.{}", b64::kodla_urlsafe(tahrif.as_bytes()), imza_b64);
        assert_eq!(
            bilet_dogrula(&sahte, &acik),
            Err(Hata("imza-dogrulanamadi"))
        );
    }

    #[test]
    fn eski_bilet_modul_alansiz() {
        let (imzalayan, acik) = test_anahtarlari();
        let bilet = bilet_uret(&imzalayan, r#"{"hwid":"X","until":"","iat":5}"#);
        let govde = bilet_dogrula(&bilet, &acik).unwrap();
        assert_eq!(govde.modul, None);
        assert_eq!(govde.until, "");
        assert_eq!(govde.iat_ms, Some(5));
    }

    #[test]
    fn bicim_ve_imza_errlari() {
        let (imzalayan, acik) = test_anahtarlari();
        assert!(bilet_dogrula("", &acik).is_err());
        assert!(bilet_dogrula("tek-parca", &acik).is_err());
        assert!(bilet_dogrula("a.b.c", &acik).is_err());
        // Doğru bilet, yanlış anahtar:
        let (_, baska_acik) = test_anahtarlari();
        let yabanci = ed25519_dalek::SigningKey::from_bytes(&[9u8; 32]);
        assert!(
            bilet_dogrula(&bilet_uret(&yabanci, r#"{"hwid":"X"}"#), &acik).is_err()
        );
        assert!(!baska_acik.is_empty());
        // Gövde JSON ama nesne değil:
        let dizi = bilet_uret(&imzalayan, "[1,2,3]");
        assert!(bilet_dogrula(&dizi, &acik).is_err());
    }
}
