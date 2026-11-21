//! Satılmış müşteri anahtarı: `ONEK-XXXX-XXXX-XXXX-XXXX.BİTİŞ.LİMİT.imza`.
//!
//! KDV ve LMZ aynı yapıyı kullanır (`lisans.py:1063-1126`,
//! `lucamizan/lisans/anahtar.py`). İmzalanan yük, ilk üç bölümün nokta ile
//! birleştirilmiş hâlinin UTF-8 baytlarıdır: `f"{anahtar}.{bitis}.{maxdev}"`
//! (`lisans.py:1090`).
//!
//! Biçim denetimi Python ile aynı sertliktedir: 5 tireli grup, doğru ONEK,
//! her grup 4 karakter. ALFABE DENETİ YAPILMAZ (Python da yapmaz) — imza
//! esastır; alfabe yalnız satıcı üretim aracı için `sabitler`de durur.

use ed25519_dalek::Signature;

use crate::Hata;
use crate::b64::coz as b64_coz;
use crate::imza::dogrulayici;
use crate::sabitler::{GRACE_GUN, MAKS_CIHAZ};
use crate::tarih::tarih_gun;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnahtarBilgisi {
    pub anahtar: String,
    /// `YYYY-MM-DD`. Python `anahtar_gir` bunu 23:59:59 kabul eder; burada
    /// takvim günü olarak yorumlanır (kalan_gun sözleşme dokümanında).
    pub bitis: String,
    pub maks_cihaz: u32,
    /// Bitiş - bugün. Negatifse süre aşıldı (grece içinde olabilir).
    pub kalan_gun: i64,
    /// `kalan_gun < 0`: bitiş geçti ama grece içinde; arayazda uyarılmalı.
    pub sure_asti: bool,
}

/// Anahtarı biçim + imza + süre açısından doğrular.
///
/// Grece dışına taşmış (`kalan_gun < -GRACE_GUN`) anahtar `Err("sure-doldu")`
/// döner — Python `anahtar_gir`'in "Lisans süresi dolmuş." dalının karşılığı.
pub fn imzali_anahtar_dogrula(
    metin: &str,
    onek: &str,
    acik_anahtar_b64: &str,
    bugun: &str,
) -> Result<AnahtarBilgisi, Hata> {
    let parcalar: Vec<&str> = metin.trim().split('.').collect();
    if parcalar.len() != 4 {
        return Err(Hata("anahtar-dort-parca-olmali"));
    }
    let (anahtar, bitis_txt, limit_txt, imza_b64) = (parcalar[0], parcalar[1], parcalar[2], parcalar[3]);

    let gruplar: Vec<&str> = anahtar.split('-').collect();
    if gruplar.len() != 5
        || gruplar[0] != onek
        || !gruplar[1..].iter().all(|g| g.chars().count() == 4)
    {
        return Err(Hata("anahtar-bicimi-gecersiz"));
    }

    let bitis_g = tarih_gun(bitis_txt).ok_or(Hata("bitis-tarihi-gecersiz"))?;
    let maks_cihaz: u32 = limit_txt.parse().map_err(|_| Hata("cihaz-limiti-gecersiz"))?;
    if !(1..=MAKS_CIHAZ).contains(&maks_cihaz) {
        return Err(Hata("cihaz-limiti-gecersiz"));
    }

    let imza_bayt = b64_coz(imza_b64)?;
    let imza = Signature::from_slice(&imza_bayt).map_err(|_| Hata("imza-64-bayt-degil"))?;
    let yuk = format!("{anahtar}.{bitis_txt}.{limit_txt}");
    dogrulayici(acik_anahtar_b64)?
        .verify_strict(yuk.as_bytes(), &imza)
        .map_err(|_| Hata("imza-dogrulanamadi"))?;

    let bugun_g = tarih_gun(bugun).ok_or(Hata("bugun-okunamadi"))?;
    let kalan = bitis_g - bugun_g;
    if kalan < -GRACE_GUN {
        return Err(Hata("sure-doldu"));
    }
    Ok(AnahtarBilgisi {
        anahtar: anahtar.to_string(),
        bitis: bitis_txt.to_string(),
        maks_cihaz,
        kalan_gun: kalan,
        sure_asti: kalan < 0,
    })
}

#[cfg(test)]
mod testler {
    use super::*;
    use crate::b64::kodla_urlsafe;
    use ed25519_dalek::{Signer, SigningKey};

    fn anahtar_uret(gizli: &[u8; 32], onek: &str, bitis: &str, limit: &str) -> String {
        let imzalayan = SigningKey::from_bytes(gizli);
        let anahtar = format!("{onek}-ABCD-EFGH-JKMN-PQRS");
        let yuk = format!("{anahtar}.{bitis}.{limit}");
        let imza = imzalayan.sign(yuk.as_bytes());
        format!("{yuk}.{}", kodla_urlsafe(&imza.to_bytes()))
    }

    fn acik_b64(gizli: &[u8; 32]) -> String {
        kodla_urlsafe(&SigningKey::from_bytes(gizli).verifying_key().to_bytes())
    }

    const GIZLI: &[u8; 32] = &[9u8; 32];

    #[test]
    fn gecerli_anahtar() {
        let metin = anahtar_uret(GIZLI, "KDV", "2027-06-30", "2");
        let b = imzali_anahtar_dogrula(&metin, "KDV", &acik_b64(GIZLI), "2026-11-21").unwrap();
        assert_eq!(b.anahtar, "KDV-ABCD-EFGH-JKMN-PQRS");
        assert_eq!(b.bitis, "2027-06-30");
        assert_eq!(b.maks_cihaz, 2);
        assert!(!b.sure_asti);
        // 2026-11-21 → 2027-06-30: 221 gün.
        assert_eq!(b.kalan_gun, 221);
    }

    #[test]
    fn bitisi_tahrif_edilen_anahtar_duser() {
        let metin = anahtar_uret(GIZLI, "KDV", "2027-06-30", "2");
        // süresiz gibi gösterme saldırısı: bitiş 2099, imza eski yüke ait.
        let sahte = metin.replace("2027-06-30", "2099-12-31");
        assert_eq!(
            imzali_anahtar_dogrula(&sahte, "KDV", &acik_b64(GIZLI), "2026-11-21"),
            Err(Hata("imza-dogrulanamadi"))
        );
    }

    #[test]
    fn grece_icinde_uyarir_disarida_duser() {
        let metin = anahtar_uret(GIZLI, "KDV", "2026-11-10", "1");
        let acik = acik_b64(GIZLI);
        // 11 gün geçmiş: grece (14) içinde, sure_asti=True.
        let b = imzali_anahtar_dogrula(&metin, "KDV", &acik, "2026-11-21").unwrap();
        assert!(b.sure_asti);
        assert_eq!(b.kalan_gun, -11);
        // 30 gün geçmiş: grece dışı.
        assert_eq!(
            imzali_anahtar_dogrula(&metin, "KDV", &acik, "2026-12-10"),
            Err(Hata("sure-doldu"))
        );
    }

    #[test]
    fn bicim_errleri() {
        let acik = acik_b64(GIZLI);
        let metin = anahtar_uret(GIZLI, "KDV", "2027-06-30", "2");
        // üç parça:
        assert!(imzali_anahtar_dogrula("a.b.c", "KDV", &acik, "2026-11-21").is_err());
        // grup uzunluğu: biçim denetimi imzadan ÖNCE gelir.
        assert_eq!(
            imzali_anahtar_dogrula(
                &metin.replace("ABCD", "ABC"),
                "KDV", &acik, "2026-11-21"
            ),
            Err(Hata("anahtar-bicimi-gecersiz"))
        );
        // ONEK uyuşmaz: LMZ anahtarı KDV olarak doğrulanamaz ve tersi:
        let lmz = anahtar_uret(GIZLI, "LMZ", "2027-06-30", "2");
        assert_eq!(
            imzali_anahtar_dogrula(&lmz, "KDV", &acik, "2026-11-21"),
            Err(Hata("anahtar-bicimi-gecersiz"))
        );
        assert!(imzali_anahtar_dogrula(&lmz, "LMZ", &acik, "2026-11-21").is_ok());
        // cihaz limiti 1-5 dışı:
        let sinir = anahtar_uret(GIZLI, "KDV", "2027-06-30", "9");
        assert_eq!(
            imzali_anahtar_dogrula(&sinir, "KDV", &acik, "2026-11-21"),
            Err(Hata("cihaz-limiti-gecersiz"))
        );
    }
}
