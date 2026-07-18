# Kglance (Oxiview)

Cua so xem truoc file sieu toc cho KDE Plasma 6. Ho tro 2 che do: **Daemon** (chay ngam, DBus) va **Standalone** (xem truc tiep, khong can DBus).

## Tinh nang

- Xem truoc ma nguon voi highlight cu phap (syntect)
- Xem truoc hinh anh (PNG, JPEG, WebP, GIF, BMP) va SVG
- Xem thong tin PDF (so trang)
- Xem cau truc file luu tru (ZIP, Tar, 7z) dang cay thu muc
- Xem noi dung thu muc
- Che do Daemon: Hien/An cua so tuc thoi, chay ngam khong tat
- Che do Standalone: Mo file truc tiep, thoat khi dong cua so
- Tich hop phim Space trong Dolphin (KIO Service Menu)
- Tu dong khoi dong cung he thong (Autostart)
- Tu dong fallback: neu Daemon chua chay, tu chuyen sang Standalone

## Yeu cau

- KDE Plasma 6 (Wayland hoac X11)
- Rust 1.85+ (edition 2024)
- Thu vien he thong: `libfontconfig`, `libfreetype`, `libxkbcommon`

## Cai dat

### Build tu ma nguon

```bash
cargo build --release
```

Mac dinh bao gom ho tro 7z. De build khong co 7z:

```bash
cargo build --release --no-default-features
```

Binary duy nhat: `target/release/kglance`

### Cai dat Autostart (che do Daemon)

```bash
cp data/kglance-daemon.desktop ~/.config/autostart/
# Dam bao duong dan Exec tro dung binary kglance
```

### Cai dat KIO Service Menu (tich hop Dolphin)

```bash
mkdir -p ~/.local/share/kio/servicemenus
cp data/kglance-rust.desktop ~/.local/share/kio/servicemenus/
```

Sau do dang xuat va dang nhap lai (hoac chay `kquitapp6 dolphin && dolphin`).

## Su dung

```bash
# Khoi dong Daemon (chay ngam, lang nghe DBus)
kglance daemon

# Xem file (tu dong chon che do)
kglance /path/to/file

# Ep che do Standalone (khong can DBus)
kglance --standalone /path/to/file
```

**Trong Dolphin (sau khi cai dat KIO Service Menu):** Chon file, nhan **Space**.

### An cua so

- **Daemon mode:** Nhan Esc hoac click nut **X** -- cua so an, tien trinh van chay.
- **Standalone mode:** Nhan Esc hoac click nut **X** -- thoat ung dung.

## Kien truc

```
Kglance/
├── src/
│   ├── main.rs            # Entrypoint (dispatch Daemon / Standalone)
│   ├── parser/            # Module phan tich dinh dang file
│   │   ├── text.rs        # Text + ma nguon (syntect)
│   │   ├── image.rs       # Hinh anh (image crate)
│   │   ├── svg.rs         # SVG -> PNG (resvg)
│   │   ├── pdf.rs         # PDF metadata (lopdf)
│   │   ├── archive.rs     # Zip/Tar/7z tree list
│   │   └── folder.rs      # Thu muc (std::fs)
│   ├── ui/
│   │   ├── mod.rs         # Cau noi Slint Window
│   │   └── window.slint   # Giao dien Slint
│   └── dbus/
│       └── service.rs     # DBus interface (zbus)
└── data/
    ├── kglance-daemon.desktop  # Autostart
    └── kglance-rust.desktop    # KIO Service Menu
```

- **Che do Daemon:** 2 luong (zbus + Slint), giao tiep qua `mpsc::channel`. Cua so an/hien, khong tat.
- **Che do Standalone:** Parse va show truc tiep trong Slint event loop. Thoat khi dong cua so.
- DBus: `org.mintori.Kglance` tren Session Bus.

## Dinh dang ho tro

| Loai     | Dinh dang                                                        |
| -------- | ---------------------------------------------------------------- |
| Ma nguon | rs, py, js, ts, jsx, tsx, html, css, json, md, toml, yaml, sh, c, cpp, go, java, ... |
| Hinh anh | PNG, JPEG, WebP, GIF, BMP, SVG                                  |
| Tai lieu | PDF (thong tin)                                                  |
| Luu tru  | ZIP, Tar, GZ, TGZ, XZ, TXZ, 7z                                  |
| Khac     | Folder, Plain text (fallback)                                    |

Gioi han dung luong: 100MB.

## Phat trien

```bash
# Kiem tra code
cargo clippy --all-targets --all-features -- -D warnings

# Chay test
cargo test

# Build release
cargo build --release

# Chay Daemon (debug)
cargo run -- daemon

# Xem file (debug)
cargo run -- /path/to/file

# Xem file standalone (debug)
cargo run -- --standalone /path/to/file
```

## Ghi chu

- Project dang o giai doan phat trien ban dau
- PDF rendering chua duoc ho tro (chi hien thi so trang)
- Media file (audio/video) chua duoc ho tro
