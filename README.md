# muhasebe-lisans

Muhasebe Paketi ekosisteminin **lisans/deneme karar cekirdegi**. Muhasebe
Paketi'ndeki Python satiri (`lisans.py`, `lisans_cekirdek.py`) ile ayni
sozlesmeyi konu edinen Rust karsiligidir; native Rust urunleri (Finansal
Hesaplayici, Ofis Kasasi) kapilarini buna dayandirir.

Baglayici sozlesme (dosya yollari, HWID tarifleri, bilet/anahtar formatlari,
degismezler): **`Muhasebe Paketi` reposu, `docs/LISANS_SOZLESMESI.md`**.
Kod ile o dokuman cakisirsa dokuman yanlistir (once testlere bak).

## Ne yapar

- `bilet::bilet_dogrula` — sunucunun (`*.workers.dev`) Ed25519 imzali
  `b64url(govde_json).b64url(imza)` biletini dogrular, govdeyi cozumler.
- `anahtar::imzali_anahtar_dogrula` — musterinin satin aldigi
  `ONEK-XXXX-XXXX-XXXX-XXXX.BITIS.LIMIT.imza` anahtarini dogrular
  (KDV ve LMZ ayni yapiyi kullanir).
- `karar` — deneme/kalan gun/bilet/kilit-metni kararlarl. Python
  `rust/lisans_hizli` (pyo3) eklentisiyle birebir ayni kurallar; tarih
  aritmetigi chrono'suz (Hinnant).
- `kimlik` — cihaz kimligi (`ONEK-` + sha256[-16:] 4'luk grup) ve deneme
  satir anahtari (`DENEME-` + sha256[-12:]).
- `sabitler` — Python ile paylasilan sabitler ve GOMULU ACIK ANAHTARLAR
  (bunlar zaten her dagitilan ikiliye gomuludur; gizli degildir).

## Ne yapMAZ

- Imza **uretemez** (gizli anahtar yok ve olmayacak; satici araclari
  repo-disinda durur).
- Disk/kayit defteri/ag erisimi yok: girdi alir, karar donerur. Cagiran
  uygulamaya bagli olan durumsal kirilmalar (sqlite, lease dosyalari, env
  kapilari) crate'in disindadir — sozlesme dokumaninda modullere yazilir.

## Kullanim

```rust
use muhasebe_lisans::{bilet_dogrula, sabitler};

let govde = bilet_dogrula(&ticket, sabitler::PAKET_YETKI_ACIK_ANAHTARI)?; // Err => yok say
```

## Test

`cargo test` — tarih gidip-gelir, imza kurallari (gecerli/tahrif edilmis),
grece-astimi, altin vektorler (canli `lisans_hizli.pyd` ile uretildi, bkz.
`tests/sozlesme.rs`), gomulu acik anahtarlarin gecerliligi.
