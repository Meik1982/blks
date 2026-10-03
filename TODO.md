# blks Roadmap & Aufgabenliste

## Status & Abgeschlossene Meilensteine

- [x] **Workspace-Architektur & Core Engine (`crates/blks-core`)**
  - 64-Bit ARX-Kompressionsfunktion mit 12 vollen Runden und Domain-Separation (`BLKS_384`).
  - 384-Bit (48-Byte) Digest mit 192-Bit theoretischer Kollisionsresistenz ($2^{192}$).
  - Zero-Dependency, Padding-freie 64-Zeichen Base64-Kodierung (Standard & URL-safe) sowie 96-Zeichen Hex-Format.
  - Page-alignierte 4 KiB Chunking-Architektur für minimale Latenz und NVMe/Page-Cache-Ausrichtung.
  - Parallele Merkle-Baumreduktion via `rayon` Work-Stealing mit adaptiver Layer-Parallelisierung.
  - Deterministischer Streaming-Hasher (`BlksHasher`) mit 100 % bit-exakter Parität zur parallelen Slice-Berechnung.
  - Zero-Copy File-Mapping via `memmap2` mit **2,15 GB/s Durchsatz** auf NVMe/RAM (464 ms pro 1 GB).

- [x] **Code-Audit & Performance-Härtung (Audit 10/2026)**
  - Bounds-Check-freie Deserialisierung via modernem Rust `as_chunks::<8>()` / `as_chunks::<2>()`.
  - Aggressives Inlining (`#[inline(always)]`) auf G-Funktion, Block-Kompression und Parent-Combiner.
  - Heap-Allokationen bei Base64/Hex durch Stack-Puffer eliminiert; statische `const REV_TABLE: [u8; 256]` zur Compile-Zeit.
  - C-FFI Panic-Safety: Vollständige Kapselung aller C-Entrypoints via `std::panic::catch_unwind`.
  - 128-KiB Streaming-Puffer für maximale Linux-Pipe-Sättigung.
  - Target-CPU Native-Konfiguration in `.cargo/config.toml` (AVX2/BMI2 RORX-Instruktionen).

- [x] **Kryptoanalytische Testsuite & Golden Vectors**
  - Feste Known-Good Testvektoren (`crates/blks-core/tests/test_vectors.rs`) für leere Eingaben, Blöcke, Chunks und Multi-Chunks.
  - Strict Avalanche Criterion (SAC) Verifikation (~50,1 % Bit-Diffusion bei 1-Bit Mutation).
  - Fragmentierte Stream-Feed-Tests über ungerade Puffergrenzen (1 B bis 8.192 B).
  - Thread-Invarianz-Tests über 1, 2, 4 und 8 Worker-Threads.
  - 100 % Testabdeckung mit 17 Unit-, Integrations- und C-FFI-Tests.
  - Strikte Zero-Warning-Policy unter `cargo clippy --all --tests -- -D warnings`.

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

- [ ] **Integration in `blkcp`:**
  - Anbindung von `blks` über die C-FFI-Schnittstelle (`include/blks.h` und `libblks_core.a`) als neuer Hashing-Algorithmus `--hash=blks` / `--blks`.
  - Nutzung der baumbasierten Eigenschaft für paralleles Multi-Ring io_uring Sharding (`-j N`) ohne Fallback auf Single-Ring!
- [ ] **AVX-512 SIMD Intrinsics:**
  - Explizite AVX-512 Vektorisierung (4 Chunks parallel in 512-Bit ZMM-Registern) zur weiteren Steigerung auf > 5 GB/s.
- [ ] **Arch Linux / CachyOS PKGBUILD:**
  - Bereitstellung in `dist/archlinux/PKGBUILD` für native Paketierung via `makepkg -si`.
- [ ] **Shell-Completions:**
  - Bash-, Zsh- und Fish-Autovervollständigung für das `blks`-Binary.
