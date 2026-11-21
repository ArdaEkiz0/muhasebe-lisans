//! Cihaz kimliği ve deneme satırı anahtarı — üretim tarifleri.
//!
//! Python `lisans.py:837-850` (cihaz_id) ve `:879-886` (deneme anahtarı) ile
//! birebir. TOHUMUN NE OLDUĞU buranın işi değil (recipe sözleşme dokumanında):
//! - Paket dünyası (kabuk + paket içi modüller): `MachineGuid | diskSeri`
//!   (kabuğun "yeni" tarifi) ve ONEK `"KDV"` — kabukla AYNI kimlik üretilmeli,
//!   yoksa bilet sahiplik kontrolü tutmaz.
//! - Ayakta duran kendi başına ürünler: kendi ONEK'ları ve kendi tohum
//!   zincirleri (bkz. docs/LISANS_SOZLESMESI.md §10).

use sha2::{Digest, Sha256};

fn sha256_hex(veri: &str) -> String {
    let ozet = Sha256::digest(veri.as_bytes());
    ozet.iter().map(|b| format!("{b:02x}")).collect()
}

/// `ONEK-XXXX-XXXX-XXXX-XXXX` — sha256(tohum) onaltılığının SON 16 hanesi,
/// büyük harf, 4'lük gruplar. Python `_cihaz_id` aynası.
pub fn cihaz_id(tohum: &str, onek: &str) -> String {
    let ozet = sha256_hex(tohum);
    let kuyruk = &ozet[ozet.len() - 16..]; // onaltılık ASCII, dilim güvenli
    let buyuk = kuyruk.to_uppercase();
    format!(
        "{onek}-{}-{}-{}-{}",
        &buyuk[0..4],
        &buyuk[4..8],
        &buyuk[8..12],
        &buyuk[12..16]
    )
}

/// Deneme satırı anahtarı: `DENEME-` + sha256(tohum)[-12:] (küçük harf).
/// Python `lisans.py:879-886` aynası; paket.db `moduller.modul` kolonu da
/// modül başına benzer bir anahtar taşır — çağıran hangi tohumu kullanacağını
/// sözleşmeden bilir.
pub fn deneme_anahtari(tohum: &str) -> String {
    let ozet = sha256_hex(tohum);
    format!("DENEME-{}", &ozet[ozet.len() - 12..])
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn sha256_bilinen_vektor() {
        // sha256("abc") = ba7816bf...f20015ad (FIPS dokümanı vektörü)
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn cihaz_id_abc() {
        // son 16: b410ff61f20015ad → büyük harf, 4'lük grup
        assert_eq!(cihaz_id("abc", "KDV"), "KDV-B410-FF61-F200-15AD");
        assert_eq!(cihaz_id("abc", "FH"), "FH-B410-FF61-F200-15AD");
    }

    #[test]
    fn deneme_anahtari_abc() {
        assert_eq!(deneme_anahtari("abc"), "DENEME-ff61f20015ad");
    }
}
