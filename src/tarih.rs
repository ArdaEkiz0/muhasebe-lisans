//! Gregoryen takvim aritmetiği — chrono yok, Hinnant'ın civil-from-days algoritması.
//!
//! `Muhasebe Paketi/rust/lisans_hizli/src/lib.rs` (pyo3) ile BAYT BAYT aynı
//! mantıktır; iki kopyayı tek kaynakta birleştirmek o modül de bu crate'e
//! taşındığında mümkün olacak (roadmap). O zamana dek eşitlik altın vektör
//! testleriyle (`tests/sozlesme.rs`) kilitli.

use crate::Hata;

pub fn artik_yil(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn aydaki_gun(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if artik_yil(y) { 29 } else { 28 }
        }
        _ => 0,
    }
}

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// `YYYY-MM-DD` (ilk 10 karakter) → 1970'den bu yana gün sayısı.
/// Python `datetime.strptime(metin[:10], "%Y-%m-%d")` davranışını izler:
/// geçersiz gün/ay/yıl → `None`.
pub fn tarih_gun(metin: &str) -> Option<i64> {
    let dilim: String = metin.chars().take(10).collect();
    let parcalar: Vec<&str> = dilim.split('-').collect();
    if parcalar.len() != 3 {
        return None;
    }
    let y: i64 = parcalar[0].parse().ok()?;
    let m: i64 = parcalar[1].parse().ok()?;
    let d: i64 = parcalar[2].parse().ok()?;
    if !(1..=9999).contains(&y) || !(1..=12).contains(&m) {
        return None;
    }
    if d < 1 || d > aydaki_gun(y, m) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

/// Gün sayısını `YYYY-MM-DD` metnine çevirir.
pub fn gun_tarih(gun: i64) -> String {
    let (y, m, d) = civil_from_days(gun);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Bitiş tarihine kalan gün (bugün bitiyorsa 0, geçmişse negatif).
/// `bugun_gun` önceden çözülmüş gün sayısıdır. Okunamayan bitişte Python
/// yedeği gibi 0 döner — "okunamayan tarih" asla genişleme üretmesin.
pub fn kalan_gun_gune(bitis: &str, bugun_gun: i64) -> i64 {
    match tarih_gun(bitis) {
        Some(hedef) => hedef - bugun_gun,
        None => 0,
    }
}

/// `kalan_gun(bitis, bugun)` — Python `lisans_hizli.kalan_gun` aynası:
/// `bugun` okunamazsa hata, `bitis` okunamazsa 0.
pub fn kalan_gun(bitis: &str, bugun: &str) -> Result<i64, Hata> {
    let bugun_g = tarih_gun(bugun).ok_or(Hata("bugun-okunamadi"))?;
    Ok(kalan_gun_gune(bitis, bugun_g))
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn tarih_gidip_gelir() {
        for (y, m, d) in [(1970, 1, 1), (2026, 11, 14), (2000, 2, 29), (1999, 12, 31)] {
            let g = days_from_civil(y, m, d);
            assert_eq!(civil_from_days(g), (y, m, d));
        }
    }

    #[test]
    fn gecersiz_tarih_yok() {
        assert!(tarih_gun("2026-02-30").is_none());
        assert!(tarih_gun("2026-13-01").is_none());
        assert!(tarih_gun("boş").is_none());
        assert!(tarih_gun("2026-02-28").is_some());
    }

    #[test]
    fn saat_damgasi_ilk_10_karakter() {
        // ZAMAN_BICIMI "YYYY-MM-DD HH:MM:SS" de gelir; ilk 10 karakter kazanır.
        assert_eq!(tarih_gun("2026-11-21 13:45:00"), tarih_gun("2026-11-21"));
    }

    #[test]
    fn kalan_gun_kurallari() {
        assert_eq!(kalan_gun("2026-11-21", "2026-11-21").unwrap(), 0);
        assert_eq!(kalan_gun("2026-11-20", "2026-11-21").unwrap(), -1);
        assert_eq!(kalan_gun("bozuk", "2026-11-21").unwrap(), 0);
        assert!(kalan_gun("2026-11-21", "bozuk").is_err());
    }
}
