# blks Roadmap & Aufgabenliste

## Status & Abgeschlossene Meilensteine

- [x] **Workspace-Architektur & Core Engine (`crates/blks-core`)**
  - 64-Bit ARX-Kompressionsfunktion mit 12 vollen Runden und Domain-Separation (`BLKS_384`).
  - 384-Bit (48-Byte) Digest mit 192-Bit theoretischer Kollisionsresistenz ($2^{192}$).
  - Zero-Dependency, Padding-freie 64-Zeichen Base64-Kodierung (Standard & URL-safe) sowie 96-Zeichen Hex-Format.
  - Page-alignierte 4 KiB Chunking-Architektur für minimale Latenz und NVMe/Page-Cache-Ausrichtung.
  - Parallele Merkle-Baumreduktion via `rayon` Work-Stealing.
  - Deterministischer Streaming-Hasher (`BlksHasher`) mit 100 % bit-exakter Parität zur parallelen Slice-Berechnung.
  - Zero-Copy File-Mapping via `memmap2` mit > 2 GB/s Durchsatz auf NVMe/RAM.

- [x] **C-FFI Kompatibilität (`include/blks.h` & `crates/blks-core/src/ffi.rs`)**
  - C-kompatible Header-Datei `include/blks.h`.
  - C-kompatible One-Shot- und Streaming-Funktionen (`blks_hash_buffer`, `blks_hasher_new`, `blks_hasher_update`, `blks_hasher_finalize`, `blks_digest_to_base64`).
  - E2E-Validierungstest in C (`tests/test_c_ffi.c`) erfolgreich integriert.

- [x] **CLI-Werkzeug (`crates/blks-cli`)**
  - Multithreaded CLI `blks` mit Stdin-Pipes und Datei-Argumenten.
  - Prüfsummen-Verifikationsmodus (`-c` / `--check`).
  - Format-Auswahl (`--base64`, `--base64-url`, `--hex`, `--raw`).
  - Thread-Begrenzung (`-j` / `--threads`).
  - Strukturierte JSON-Telemetrie (`--json`).
  - Benchmark-Modus (`--benchmark`) mit Durchsatzanzeige in GB/s.

---

## Nächste Schritte & Zukünftige Optimierungspotenziale

- [ ] **AVX2 / AVX-512 SIMD Intrinsics:**
  - Vektorisierte 4-fach bzw. 8-fach parallele G-Funktions-Berechnung für 4 Chunks gleichzeitig auf x86_64 zur weiteren Steigerung des Durchsatzes auf > 5 GB/s pro Core.
- [ ] **Integration in `blkcp`:**
  - Anbindung von `blks` über die C-FFI-Schnittstelle (`include/blks.h` und `libblks_core.a`) als neuer Hashing-Algorithmus `--hash=blks` / `--blks`.
  - Nutzung der baumbasierten Eigenschaft für paralleles Multi-Ring io_uring Sharding (`-j N`) ohne Fallback auf Single-Ring!
- [ ] **Arch Linux / CachyOS PKGBUILD:**
  - Bereitstellung in `dist/archlinux/PKGBUILD` für native Paketierung via `makepkg -si`.
- [ ] **Shell-Completions:**
  - Bash-, Zsh- und Fish-Autovervollständigung für das `blks`-Binary.
