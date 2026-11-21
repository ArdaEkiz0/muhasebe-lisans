//! Muhasebe Paketi lisans/deneme karar çekirdeği.
//!
//! Bağlayıcı sözleşme: `Muhasebe Paketi` reposu, `docs/LISANS_SOZLESMESI.md`.
//! Bu crate Python satırının (`lisans.py`, `lisans_cekirdek.py`,
//! `rust/lisans_hizli` pyo3 eklentisi) Rust karşılığıdır ve aynı kuralları
//! uygular: girdi alır, karar döner. Disk, kayıt defteri ve ağ erişimi YOK;
//! imza üretmek de yok — yalnızca doğrulama.
//!
//! Güvenlik modeli: kodun gizliliğine değil Ed25519 imzasına dayanır.
//! Gömülü açık anahtarlar (`sabitler`) zaten her dağıtılan ikilide bulunur;
//! gizli anahtar yalnız sunucudadır.

#![forbid(unsafe_code)]

pub mod b64;
pub mod bilet;
pub mod imza;
pub mod kimlik;

pub mod anahtar;
pub mod karar;
pub mod sabitler;
pub mod tarih;

pub use anahtar::{AnahtarBilgisi, imzali_anahtar_dogrula};
pub use bilet::{Govde, bilet_dogrula};
pub use karar::{
    BiletDurumu, BiletKarari, DenemeDurumu, DenemeKarari, bilet_karari, deneme_karari,
    kilit_metni,
};
pub use kimlik::{cihaz_id, deneme_anahtari};
pub use tarih::{gun_tarih, kalan_gun, tarih_gun};

use std::fmt;

/// Doğrulama/ayrıştırma hatası. `0.1.0`'da sözlük sabittir; mesajı kullanıcıya
/// göstermek için değil, günlüğe yazmak için tasarlandı. Kullanıcıya gösterilecek
/// metin `karar::kilit_metni` ve çağıran uygulamanın kendi sözlüğündedir.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hata(pub &'static str);

impl fmt::Display for Hata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for Hata {}

/// Derleme bilgisi; tanılama için.
pub fn surum() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
