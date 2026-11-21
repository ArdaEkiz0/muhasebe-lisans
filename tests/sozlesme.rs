//! Sözleşme testleri — altın vektörler CANLI Python tarafından üretildi.
//!
//! `rust/lisans_hizli` (pyo3, Muhasebe Paketi deposu) derlenmiş .pyd'i
//! `.venv-derle` Python'u ile çalıştırılıp aynı girdiler için çıktı alındı;
//! aşağıdaki beklenen değerler odur (21 Kas 2026). Yani bu crate, müşterinin
//! bugün kullandığı karar motoruyla aynı kararı verdiğini kanıtlar.
//! Yenir üretme komutu bu dosyanın sonundaki yorumda.

use muhasebe_lisans::karar::{DenemeDurumu, deneme_karari};

#[test]
fn deneme_karari_altin_vektorler() {
    // (ilk_deneme, kurulum, bugun, deneme_gun, satir_var, durum, bitis, kalan)
    let vektorler: [(Option<&str>, Option<&str>, &str, i64, bool, &str, Option<&str>, i64); 8] = [
        (None, None, "2026-11-21", 15, false, "baslamamis", None, 15),
        (None, Some("2026-11-21"), "2026-11-21", 15, false, "baslamamis", None, 15),
        (Some("2026-11-18"), Some("2026-11-21"), "2026-11-21", 15, true, "deneme", Some("2026-12-03"), 12),
        (Some("2026-11-21"), Some("2026-11-11"), "2026-11-21", 15, true, "deneme", Some("2026-11-26"), 5),
        (Some("2027-01-01"), None, "2026-11-21", 15, true, "deneme", Some("2026-12-06"), 15),
        (Some("2026-10-01"), Some("2026-10-01"), "2026-11-21", 15, true, "kilitli", Some("2026-10-16"), -36),
        (Some("2026-11-06"), None, "2026-11-21", 15, true, "deneme", Some("2026-11-21"), 0),
        (Some("2026-11-07"), None, "2026-11-21", 15, true, "deneme", Some("2026-11-22"), 1),
    ];
    for (ilk, kur, bugun, gun, satir, durum, bitis, kalan) in vektorler {
        let k = deneme_karari(ilk, kur, bugun, gun, satir).unwrap();
        assert_eq!(k.durum.as_str(), durum, "vektör: {ilk:?} {kur:?} {bugun}");
        assert_eq!(k.bitis.as_deref(), bitis, "vektör bitiş: {ilk:?} {kur:?}");
        assert_eq!(k.kalan_gun, kalan, "vektör kalan: {ilk:?} {kur:?}");
    }
}

#[test]
fn kalan_gun_altin_vektorleri() {
    use muhasebe_lisans::kalan_gun;
    assert_eq!(kalan_gun("2026-12-03", "2026-11-21").unwrap(), 12);
    assert_eq!(kalan_gun("2026-11-21", "2026-11-21").unwrap(), 0);
    assert_eq!(kalan_gun("2026-10-16", "2026-11-21").unwrap(), -36);
    assert_eq!(kalan_gun("", "2026-11-21").unwrap(), 0);
    assert_eq!(kalan_gun("2026-11-21T13:00", "2026-11-21").unwrap(), 0);
}

#[test]
fn bilet_karari_altin_vektorleri() {
    use muhasebe_lisans::karar::bilet_karari;
    // (modul, bilet_modul, hwid, kabul_listesi, until, sonuc-durumu, kalan)
    // None sonuç = geçersiz bilet.
    assert!(
        bilet_karari("kdv", None, "KDV-AA", &["KDV-AA"], Some("2027-01-01"), "2026-11-21")
            .unwrap()
            .is_some()
    );
    assert!(
        bilet_karari("sgk", None, "KDV-AA", &["KDV-AA"], Some("2027-01-01"), "2026-11-21")
            .unwrap()
            .is_none(),
        "modul alanı olmayan eski bilet yalnız kdv'yi açmalı"
    );
    let k = bilet_karari("sgk", Some("sgk"), "H", &["H"], Some("2026-11-01"), "2026-11-21").unwrap();
    let k = k.unwrap();
    assert_eq!(k.durum.as_str(), "kilitli");
    assert_eq!(k.kalan_gun, Some(-20));
    let aktif = bilet_karari("mizan", Some("Mizan"), "H", &["H"], Some("2026-11-21"), "2026-11-21")
        .unwrap()
        .unwrap();
    assert_eq!(aktif.durum.as_str(), "aktif"); // büyük/küçük harf toleransı trim+lower
    assert_eq!(aktif.kalan_gun, Some(0));
}

#[test]
fn deneme_durumu_sozi_metinleri() {
    assert_eq!(DenemeDurumu::Baslamamis.as_str(), "baslamamis");
    assert_eq!(DenemeDurumu::Deneme.as_str(), "deneme");
    assert_eq!(DenemeDurumu::Kilitli.as_str(), "kilitli");
}

// Altın vektörleri yeniden üretmek için (Muhasebe Paketi kökünde):
//   1) rust\lisans_hizli\target\release\lisans_hizli.dll dosyasını bir klasöre
//      "lisans_hizli.pyd" olarak kopyala;
//   2) .venv-derle\Scripts\python.exe ile:
//      import lisans_hizli as l
//      l.deneme_karari(modul, ilk_deneme, kurulum, bugun, deneme_gun, satir_var)
//      l.kalan_gun(bitis, bugun); l.bilet_karari(...)
//   3) çıktıları yukarıdaki vektörlerle karşılaştır; fark VARSA önce Python
//      testlerine (test_lisans_cekirdek_native.py) bak, sonra burada düzelt.
