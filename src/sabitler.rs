//! Python ile paylaşılan sabitler — TEK kaynak sözleşmenin bu yüzü.
//!
//! Bir sabiti buradan değiştirmek, Python tarafında da (`lisans.py`,
//! `lisans_cekirdek.py`, worker) aynı anda değişmedikçe YAYINLAMAYI
//! BOZAR; ikisi birlikte hareket eder. Kaynak satırları yorumda.

/// Deneme süresi, gün. `lisans_cekirdek.py:51` / worker `DEMO_DAYS`.
pub const DENEME_GUN: i64 = 15;

/// Sunucu hiç görmemiş kurulumun çevrimdışı çalışma hakkı, gün. `lisans.py:44`.
pub const CEVRIMDISI_GUN: i64 = 3;

/// Satılmış anahtar için bitişten sonraki tolerans, gün. `lisans.py:45`.
pub const GRACE_GUN: i64 = 14;

/// Anahtar başına en çok cihaz. `lisans.py:40`.
pub const MAKS_CIHAZ: u32 = 5;

/// Saat sapma toleransı, saniye. `lisans.py:186`.
pub const SAAT_SAPMA_TOLERANS_SN: i64 = 1800;

/// Satıcı anahtar üretim alfabesi (I, O, 0, 1 yok). `lisans.py:1242-1245`.
/// DOĞRULAMA bunu kontrol ETMEZ (Python da etmez): biçim + imza esastır;
/// alfabe yalnız satıcı araçları içindir.
pub const ANAHTAR_ALFABE: &str = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

// --- Gömülü açık anahtarlar -------------------------------------------------
// Bunlar GİZLİ DEĞİLDİR: zaten her dağıtılan uygulamanın içine gömülüdür.
// Gizli olan yalnız sunucudaki imzalama anahtarıdır ve bu crate'e giremez.

/// KDV satılmış anahtar imzası. `lisans.py:51` (PUBLIC_KEY_B64).
pub const KDV_SATIS_ACIK_ANAHTARI: &str = "JvSSAsBFgPbVQpyLhCFV23AzDpRlk3OjVJbttlI209w";

/// Paket yetki/modül bileti imzası. `lisans.py:320` (AUTH_PUBLIC_KEY_B64).
pub const PAKET_YETKI_ACIK_ANAHTARI: &str = "EvVRjtCn4wMDAtp5oxeL3C+SRIgW91J6DLVSclZyJfc=";

/// E-Kesinti lease/manifest imzası. `ekesinti/config.py:14` (LICENSE_PUBLIC_KEY).
pub const EKESINTI_ACIK_ANAHTARI: &str = "GYWjDCKeOqv12vi7KnUVpi_3Nd2IaICdNTeReqQIQGM";

/// Luca Mizan anahtar imzası. `lucamizan/lisans/anahtar.py:30` (ACIK_ANAHTAR_B64).
pub const LMZ_ACIK_ANAHTARI: &str = "b5olvsQ3-tdt95rKw0EH10NDYSaSgD3Num9qcnvOUz8";
