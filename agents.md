# WinDebloat for Linux - AI Agent Rehberi

Bu dosya, AI asistanlarının bu projeyi anlaması, geliştirmesi ve bakımını yapması için hazırlanmıştır.

---

## Proje Hakkında

**WinDebloat for Linux**, Linux sistemler için geliştirilmiş gelişmiş bir debloating (gereksiz dosya temizleme) ve bakım aracıdır. Rust dilinde yazılmış olup, Ratatui kütüphanesi ile güzel bir Terminal User Interface (TUI) sunar.

### Temel Özellikler

- **Çoklu distro desteği**: Debian/Ubuntu, Arch/Manjaro, Fedora/RHEL, openSUSE, Alpine
- **7+ temizleme modülü**: Packages, System, Apps, Privacy, Services, Duplicates, Disk, Containers
- **Tam yedekleme/geri yükleme sistemi**
- **Güvenlik öncelikli**: Safe mode, dry-run, onay dialogları, whitelist/blacklist
- **CLI ve TUI modları**

---

## Proje Yapısı

```
WinDebloat-ForLinux/
├── src/
│   ├── main.rs              # Ana giriş noktası, CLI komut işleyicileri
│   ├── app.rs               # Ana App yapısı, TUI event loop
│   ├── cli.rs               # Clap tabanlı CLI argüman tanımları
│   │
│   ├── config/              # Konfigürasyon sistemi
│   │   ├── mod.rs
│   │   ├── loader.rs        # Config dosyası yükleme/kaydetme
│   │   └── schema.rs        # Config struct tanımları
│   │
│   ├── core/                # Çekirdek işlevler
│   │   ├── mod.rs
│   │   ├── scanner.rs       # Tarama ve CleanItem/CleanResult yapıları
│   │   ├── cleaner.rs       # Temizleme işlemleri (silme, backup)
│   │   ├── backup.rs       # Yedekleme/geri yükleme sistemi
│   │   └── logger.rs        # Eylem günlükleme
│   │
│   ├── modules/            # Temizleme modülleri (trait sistemi)
│   │   ├── mod.rs          # ModuleRegistry, CleanModule trait
│   │   ├── packages.rs     # Paket yöneticisi temizleme
│   │   ├── system.rs       # Sistem dosyaları (logs, tmp, cache)
│   │   ├── apps.rs        # Uygulama cache (browser, flatpak, docker)
│   │   ├── privacy.rs     # Gizlilik (history, clipboard)
│   │   ├── services.rs    # Servis yönetimi (systemd)
│   │   ├── duplicates.rs   # Yinelenen dosya bulma (SHA-256)
│   │   ├── disk.rs        # Disk kullanımı analizi
│   │   ├── containers.rs  # Docker/Podman temizleme
│   │   └── logs.rs        # Log dosyaları temizleme
│   │
│   ├── os/                 # OS soyutlama katmanı
│   │   ├── mod.rs
│   │   ├── distro.rs       # OSInfo, DistroFamily algılama
│   │   ├── init.rs         # Init sistemi algılama
│   │   └── pkg_managers/  # Paket yöneticisi soyutlaması
│   │       ├── mod.rs     # PkgManager dispatcher
│   │       ├── apt.rs     # Debian/Ubuntu
│   │       ├── pacman.rs  # Arch/Manjaro
│   │       ├── dnf.rs     # Fedora/RHEL
│   │       ├── zypper.rs  # openSUSE
│   │       └── apk.rs     # Alpine
│   │
│   ├── tui/                # TUI bileşenleri
│   │   ├── mod.rs
│   │   ├── ui.rs           # Ana render mantığı, ThemeColors
│   │   └── components/
│   │       ├── mod.rs
│   │       ├── sidebar.rs  # Sol menü (kategoriler)
│   │       ├── content.rs  # Ana içerik paneli (liste)
│   │       └── dialogs.rs  # Dialog pencereleri
│   │
│   └── utils/              # Yardımcı araçlar
│       ├── mod.rs
│       ├── error.rs        # AppError, Result tipi
│       ├── disk.rs         # Disk boyutu hesaplama
│       ├── formatting.rs   # Byte formatlama
│       └── permissions.rs  # Dosya izin kontrolü
│
├── Cargo.toml             # Bağımlılıklar
├── README.md              # Kullanıcı dökümantasyonu
└── agents.md              # AI Agent rehberi (bu dosya)
```

---

## Teknoloji Yığını

### Bağımlılıklar (Cargo.toml)

| Kütüphane | Sürüm | Kullanım |
|-----------|-------|----------|
| ratatui | 0.29 | TUI framework |
| crossterm | 0.28 | Terminal I/O |
| clap | 4 | CLI argüman parsing |
| serde | 1 | Serialization |
| toml | 0.8 | Config dosyası |
| sysinfo | 0.33 | Sistem bilgileri |
| chrono | 0.4 | Tarih/saat |
| uuid | 1 | Benzersiz ID |
| sha2 | 0.10 | Hash (duplicates) |
| rayon | 1.10 | Paralel işleme |
| walkdir | 2 | Dizin walking |
| dirs | 5 | Platform dizin yolları |
| thiserror | 2 | Error handling |
| anyhow | 1 | Result tipi |

---

## Kod Yapısı ve Tasarım Desenleri

### 1. CleanModule Trait Sistemi

Tüm temizleme modülleri `CleanModule` trait'ini implemente eder:

```rust
pub trait CleanModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn category(&self) -> Category;
    fn description(&self) -> &'static str;
    fn is_available(&self, _os: &OSInfo) -> bool { true }
    fn scan(&self, os: &OSInfo) -> Result<Vec<CleanItem>>;
}
```

**Yeni modül ekleme:** `src/modules/mod.rs`'deki `ModuleRegistry::new()` fonksiyonuna yeni modül eklenir.

### 2. Category Enum

Temizleme kategorileri `src/core/scanner.rs`'de tanımlı:

```rust
pub enum Category {
    Packages, System, Apps, Privacy,
    Services, Duplicates, Containers, Disk, Wizard
}
```

### 3. CleanItem Yapısı

Her temizlenecek kalem:

```rust
pub struct CleanItem {
    pub id: String,          // Benzersiz ID
    pub path: String,        // Dosya/dizin yolu
    pub size: u64,            // Boyut (byte)
    pub safe: bool,          // Güvenli mi?
    pub description: String,// Açıklama
    pub clean_command: String,// Temizleme komutu
    pub selected: bool,      // Seçili mi?
    pub category: String,    // Kategori ID
    pub age_days: Option<u64>,// Yaş (gün)
    pub file_count: Option<u64>// Dosya sayısı
}
```

### 4. OS Soyutlama

`OSInfo` yapısı distro ailesini algılar:

```rust
pub enum DistroFamily {
    Debian, Arch, Fedora, OpenSuse, Alpine,
    Void, Gentoo, Slackware, Other(String)
}
```

`PkgManager` dispatcher, aileye göre doğru modülü çağırır.

### 5. TUI Bileşen Yapısı

`AppUi` yapısı:
- `sidebar: Sidebar` - Sol menü
- `content: ContentPanel` - İçerik listesi
- `dialog: Dialog` - Aktif dialog (ConfirmClean, Progress, Report, UndoList, Search, Quit)
- `wizard: Option<DiskWizardState>` - Disk wizard durumu
- `status_bar: String` - Durum çubuğu mesajı

---

## Önemli Dosyalar ve Satırlar

### Ana Akış (main.rs)

- Satır 15-76: CLI komut switch/işleyici (`Commands` enum)
- Satır 78-137: `run_cli_scan()` - Tarama işlemi
- Satır 140-246: `run_cli_clean()` - Temizleme işlemi
- Satır 248-299: `run_cli_undo()` - Geri yükleme
- Satır 301-396: `run_cli_status()` - Sistem bilgileri

### App (app.rs)

- Satır 50-75: `App::new()` - App oluşturma
- Satır 77-105: `run_tui()` - TUI başlatma
- Satır 107-190: `tui_event_loop()` - Ana event döngüsü
- Satır 205-475: `handle_key()` - Klavye işleyici

### Config Sistemi (config/)

- `schema.rs`: Config struct'ları, default değerler
- `loader.rs`: Config yükleme, varsayılan oluşturma, path bulma

---

## CLI Komutları

```bash
# TUI başlat (varsayılan)
windebloat
windebloat tui

# Tarama
windebloat scan --all
windebloat scan --category packages --json

# Temizleme
windebloat clean --all --yes
windebloat clean --category system --dry-run

# Geri yükleme
windebloat undo --list
windebloat undo --id <backup-id>

# Sistem bilgisi
windebloat status

# Loglar
windebloat logs --count 20

# Konfigürasyon
windebloat config show
windebloat config edit
windebloat config reset
```

---

## TUI Klavye Kısayolları

| Tuş | Eylem |
|-----|-------|
| ↑/↓ | Navigasyon |
| ←/→ | Kategori değiştir |
| Space | Seçim toggle |
| Tab | Odak değiştir (liste/detay) |
| A | Analyze/scan kategori |
| S | Scan all |
| C | Clean seçili |
| T | Toggle all |
| R | Safe only |
| U | Undo/list backups |
| / | Arama/filtre |
| Q/Esc | Çıkış |
| ?/H | Yardım |

---

## Geliştirme Kuralları

### Hata Yönetimi

- `crate::utils::error::Result<T> = Result<T, AppError>` kullan
- `AppError` enum'ı `thiserror` ile tanımlı
- Önemli hatalar için `?` operatörü, kullanıcıya gösterilebilir mesaj için `user_message()` metodu

### Konvansiyonlar

1. **Modül isimleri**: snake_case (packages.rs, pkg_managers/)
2. **Struct isimleri**: PascalCase (CleanItem, ModuleRegistry)
3. **Trait isimleri**: PascalCase (CleanModule)
4. **Enum variant'ları**: PascalCase (Category::Packages)
5. **Pub alanlar**: snake_case (total_size, is_available)
6. **Fonksiyonlar**: snake_case (scan_category, is_debian_based)

### Test Kuralları

- Testler `src/` dışında `test_*.rs` olarak kök dizinde
- `cargo test` ile çalıştır
- Integration testler için `examples/` dizini

### Linting

```bash
cargo fmt        # Format
cargo clippy     # Lint
cargo check     # Syntax kontrol
```

---

## Yaygın Görevler

### 1. Yeni Temizleme Modülü Ekleme

1. `src/modules/` altında yeni `.rs` dosyası oluştur
2. `CleanModule` trait'ini implemente et
3. `src/modules/mod.rs`'de import ve `ModuleRegistry::new()`'e ekle

```rust
// Örnek modül yapısı
pub struct MyModule;

impl CleanModule for MyModule {
    fn id(&self) -> &'static str { "mymodule" }
    fn name(&self) -> &'static str { "My Module" }
    fn category(&self) -> Category { Category::System }
    fn description(&self) -> &'static str { "Açıklama" }

    fn scan(&self, os: &OSInfo) -> Result<Vec<CleanItem>> {
        // Tarama mantığı
    }
}
```

### 2. Yeni Paket Yöneticisi Ekleme

1. `src/os/pkg_managers/` altında yeni dosya oluştur
2. Fonksiyonları implemente et: `clean_cache()`, `clean_orphaned()`, `clean_old_kernels()`
3. `src/os/pkg_managers/mod.rs`'de dispatch ekle

### 3. Yeni Dialog Ekleme

1. `src/tui/components/dialogs.rs`'de `Dialog` enum'una variant ekle
2. `Dialogs` struct'ına render metodu ekle
3. `app.rs`'de `handle_key()`'ta işle

---

## Güvenlik Notları

1. **Backup her zaman aktif olmalı** (default: enabled=true)
2. **Safe mode** varsayılan olarak açık
3. **Dry-run** her temizleme öncesi gösterilmeli
4. **Root yetkisi gereken işlemler** için kullanıcı uyarılmalı
5. **Whitelist/Blacklist** dosya koruma için kullanılır

---

## Mimari Özet

```
┌─────────────────────────────────────────────────┐
│                   main.rs                       │
│              (CLI dispatcher)                   │
└─────────────────────┬───────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
   ┌────▼────┐                ┌────▼────┐
   │  CLI    │                │  App    │
   │  Mode   │                │  (TUI)  │
   └─────────┘                └────┬────┘
                                   │
        ┌──────────────────────────┼──────────────────────────┐
        │                          │                          │
   ┌────▼────────┐      ┌─────────▼──────────┐    ┌──────────▼────┐
   │ ModuleReg   │      │   TUI Components  │    │  Core Modules │
   │ (scanner)   │      │ (sidebar/content) │    │ (backup/etc)  │
   └────┬────────┘      └────────────────────┘    └───────────────┘
        │
   ┌────┴────────┐
   │   Modules   │
   ├─────────────┤
   │ packages.rs │
   │ system.rs   │
   │ apps.rs     │
   │ privacy.rs  │
   │ services.rs │
   │ duplicates.rs│
   │ disk.rs     │
   │ containers.rs│
   └─────────────┘
```

---

## Faydalı Komutlar

```bash
# Derleme
cargo build              # Debug
cargo build --release    # Release

# Test
cargo test

# Format ve lint
cargo fmt
cargo clippy -- -D warnings

# Çalıştırma
cargo run
cargo run -- --help

# Doc
cargo doc --open

# Bağımlılık analizi
cargo tree
cargo tree -i <package>
```

---

## Katkıda Bulunma

1. Fork yap
2. Feature branch oluştur (`git checkout -b feature/xyz`)
3. Değişiklikleri commit et
4. Test et ve lint geçir
5. Pull request oluştur

Testler ve lint geçmeden PR kabul edilmez.

---

*Bu dosya AI asistanları için projenin tam bir referans rehberidir. Herhangi bir soruda bu dosyaya bakılmalıdır.*