//! SAF karar mantığı — Python `rust/lisans_hizli/src/lib.rs` (pyo3) ile
//! birebir aynı kurallar. Eşitlik `tests/sozlesme.rs`'deki altın vektörlerle
//! (canlı .pyd'den üretildi) kilitlidir.

use crate::Hata;
use crate::tarih::{gun_tarih, kalan_gun_gune, tarih_gun};

/// Deneme durum makinesinin üç hâli (`lisans_cekirdek.Durum` alt kümesi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenemeDurumu {
    /// Saat henüz işlemiyor; kullanıcı "Denemeyi başlat" diyecek.
    Baslamamis,
    /// Deneme sürüyor.
    Deneme,
    /// Deneme bitti — kilit ekranı.
    Kilitli,
}

impl DenemeDurumu {
    /// Python tarafının sözleşme metinleri (DB/kayıt değerleri bunlar).
    pub fn as_str(&self) -> &'static str {
        match self {
            DenemeDurumu::Baslamamis => "baslamamis",
            DenemeDurumu::Deneme => "deneme",
            DenemeDurumu::Kilitli => "kilitli",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DenemeKarari {
    pub durum: DenemeDurumu,
    /// `baslamamis`ta None: saat yoksa bitiş tarihi de yok.
    pub bitis: Option<String>,
    pub kalan_gun: i64,
}

/// Deneme kararı — `lisans_hizli.deneme_karari` ile aynı kural:
/// taban önce kayıttaki `ilk_deneme`, yoksa kurulum günü; **kurulum daha
/// eskiyse o kazanır** (DB silinerek deneme uzatılamaz); taban gelecekteyse
/// bugüne kırpılır (saat ileri alma denemesi). Gün TAKVİM günüdür.
pub fn deneme_karari(
    ilk_deneme: Option<&str>,
    kurulum: Option<&str>,
    bugun: &str,
    deneme_gun: i64,
    satir_var: bool,
) -> Result<DenemeKarari, Hata> {
    let bugun_g = tarih_gun(bugun).ok_or(Hata("bugun-okunamadi"))?;
    let ilk = ilk_deneme.and_then(tarih_gun);
    let kur = kurulum.and_then(tarih_gun);

    let mut baslangic = ilk.or(kur).unwrap_or(bugun_g);
    if let Some(k) = kur {
        if k < baslangic {
            baslangic = k;
        }
    }
    if baslangic > bugun_g {
        baslangic = bugun_g;
    }

    let bitis_g = baslangic + deneme_gun;
    let kalan = bitis_g - bugun_g;
    let bitis = gun_tarih(bitis_g);

    if kalan < 0 {
        return Ok(DenemeKarari { durum: DenemeDurumu::Kilitli, bitis: Some(bitis), kalan_gun: kalan });
    }
    let baslamis = satir_var && ilk.is_some();
    if !baslamis {
        return Ok(DenemeKarari { durum: DenemeDurumu::Baslamamis, bitis: None, kalan_gun: kalan });
    }
    Ok(DenemeKarari { durum: DenemeDurumu::Deneme, bitis: Some(bitis), kalan_gun: kalan })
}

/// İmzası DOĞRULANMIŞ biletin modül kararı. `durum`un iki hâli Python
/// sözleşmesiyle sınırlı: "aktif" | "kilitli".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiletDurumu {
    Aktif,
    Kilitli,
}

impl BiletDurumu {
    pub fn as_str(&self) -> &'static str {
        match self {
            BiletDurumu::Aktif => "aktif",
            BiletDurumu::Kilitli => "kilitli",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiletKarari {
    pub durum: BiletDurumu,
    pub bitis: Option<String>,
    pub kalan_gun: Option<i64>,
}

/// Bilet → modül kararı; `Ok(None)` = bilet bu modül/cihaz için geçersiz
/// (çağıran denemeye düşer). İMZA DOĞRULAMASI burada değil: çağıran önce
/// `bilet::bilet_dogrula` ile doğrulamalı, sonra gövdeyi buraya vermeli.
///
/// `modul` küçük harfli beklenir ("kdv", "mizan", "sgk", ...). `modul`
/// alanı olmayan ESKİ biletler yalnız "kdv" sayılır (geriye uyum,
/// `lib.rs:173-177`).
pub fn bilet_karari(
    modul: &str,
    bilet_modul: Option<&str>,
    hwid: &str,
    hwid_kabul: &[&str],
    until: Option<&str>,
    bugun: &str,
) -> Result<Option<BiletKarari>, Hata> {
    let bugun_g = tarih_gun(bugun).ok_or(Hata("bugun-okunamadi"))?;

    if !hwid_kabul.iter().any(|k| *k == hwid) {
        return Ok(None);
    }
    let bm = bilet_modul.unwrap_or("kdv").trim().to_lowercase();
    if bm != modul {
        return Ok(None);
    }
    let until_dilim = until.unwrap_or("").trim();
    let until_kisa: String = until_dilim.chars().take(10).collect();
    if until_kisa.is_empty() {
        return Ok(Some(BiletKarari { durum: BiletDurumu::Aktif, bitis: None, kalan_gun: None }));
    }
    let kalan = kalan_gun_gune(&until_kisa, bugun_g);
    let durum = if kalan < 0 { BiletDurumu::Kilitli } else { BiletDurumu::Aktif };
    Ok(Some(BiletKarari { durum, bitis: Some(until_kisa), kalan_gun: Some(kalan) }))
}

/// Kilit ekranı metni: (başlık, açıklama). `lisans_cekirdek.kilit_mesaji`
/// aynası — müşteriye görünen strings BİREBİR değiştirilmez.
/// `kalan_gun` None iken Python f-string'i gibi kelimenin tamama "None" yazılır.
pub fn kilit_metni(durum: &str, kaynak: &str, kalan_gun: Option<i64>) -> (String, String) {
    match durum {
        "deneme" => (
            "Deneme sürümü".into(),
            format!(
                "{} gün kaldı.",
                kalan_gun.map(|k| k.to_string()).unwrap_or_else(|| "None".into())
            ),
        ),
        "kilitli" => {
            if kaynak == "deneme" {
                (
                    "Deneme süresi doldu".into(),
                    "Bu modülün 15 günlük denemesi bitti. Devam etmek için lisans alın.".into(),
                )
            } else {
                (
                    "Lisans gerekli".into(),
                    "Bu modül için geçerli bir lisans bulunamadı.".into(),
                )
            }
        }
        _ => ("Aktif".into(), "Lisans geçerli.".into()),
    }
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn deneme_baslamamis() {
        let k = deneme_karari(None, Some("2026-11-21"), "2026-11-21", 15, false).unwrap();
        assert_eq!(k.durum, DenemeDurumu::Baslamamis);
        assert_eq!(k.bitis, None);
        assert_eq!(k.kalan_gun, 15);
    }

    #[test]
    fn deneme_kosuyor() {
        let k = deneme_karari(Some("2026-11-18"), Some("2026-11-21"), "2026-11-21", 15, true).unwrap();
        assert_eq!(k.durum, DenemeDurumu::Deneme);
        assert_eq!(k.bitis.as_deref(), Some("2026-12-03"));
        assert_eq!(k.kalan_gun, 12);
    }

    #[test]
    fn kurulum_eskisi_kazanir_db_silmek_uzatmaz() {
        // ilk_deneme bugün ama kurulum 10 gün önce: taban kurulumdur.
        let k = deneme_karari(Some("2026-11-21"), Some("2026-11-11"), "2026-11-21", 15, true).unwrap();
        assert_eq!(k.bitis.as_deref(), Some("2026-11-26"));
        assert_eq!(k.kalan_gun, 5);
    }

    #[test]
    fn gelecek_taban_bugune_kirpilir() {
        let k = deneme_karari(Some("2027-01-01"), None, "2026-11-21", 15, true).unwrap();
        assert_eq!(k.durum, DenemeDurumu::Deneme);
        assert_eq!(k.kalan_gun, 15);
    }

    #[test]
    fn deneme_bitti_kilitli() {
        let k = deneme_karari(Some("2026-10-01"), None, "2026-11-21", 15, true).unwrap();
        assert_eq!(k.durum, DenemeDurumu::Kilitli);
        assert!(k.kalan_gun < 0);
    }

    #[test]
    fn bilet_hwid_eslesmezse_yok() {
        let k = bilet_karari("kdv", Some("kdv"), "KDV-AA", &["KDV-BB"], Some("2027-01-01"), "2026-11-21").unwrap();
        assert_eq!(k, None);
    }

    #[test]
    fn bilet_modul_eslesmezse_yok() {
        let k = bilet_karari("mizan", Some("kdv"), "X", &["X"], Some("2027-01-01"), "2026-11-21").unwrap();
        assert_eq!(k, None);
    }

    #[test]
    fn eski_bilet_yalniz_kdv() {
        // modul alanı yok (None) → "kdv" sayılır.
        let k = bilet_karari("kdv", None, "X", &["X"], Some("2027-01-01"), "2026-11-21").unwrap();
        assert_eq!(k.unwrap().durum, BiletDurumu::Aktif);
        let k2 = bilet_karari("mizan", None, "X", &["X"], Some("2027-01-01"), "2026-11-21").unwrap();
        assert_eq!(k2, None);
    }

    #[test]
    fn bos_until_suresiz_aktif() {
        let k = bilet_karari("kdv", Some("kdv"), "X", &["X"], Some("  "), "2026-11-21").unwrap();
        let k = k.unwrap();
        assert_eq!(k.durum, BiletDurumu::Aktif);
        assert_eq!(k.bitis, None);
        assert_eq!(k.kalan_gun, None);
    }

    #[test]
    fn until_gecmis_kilitli() {
        let k = bilet_karari("kdv", Some("kdv"), "X", &["X"], Some("2026-11-01 23:59"), "2026-11-21").unwrap();
        let k = k.unwrap();
        assert_eq!(k.durum, BiletDurumu::Kilitli);
        assert_eq!(k.bitis.as_deref(), Some("2026-11-01"));
    }

    #[test]
    fn kilit_metni_python_metinleriyle_ayni() {
        assert_eq!(
            kilit_metni("deneme", "deneme", Some(5)),
            ("Deneme sürümü".into(), "5 gün kaldı.".into())
        );
        assert_eq!(
            kilit_metni("kilitli", "deneme", None).1,
            "Bu modülün 15 günlük denemesi bitti. Devam etmek için lisans alın."
        );
        assert_eq!(
            kilit_metni("kilitli", "lisans", None).0,
            "Lisans gerekli"
        );
        assert_eq!(kilit_metni("deneme", "deneme", None).1, "None gün kaldı.");
    }
}
