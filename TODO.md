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

- [x] **Code-Audit, Hardware-Vektorisierung & Performance-Härtung (Audit 10/2026)**
  - **4-Wege AVX2 SIMD-Vektorengine:** Verarbeitet 4 Chunks parallel in 256-Bit YMM-Registern in Lockstep (`crates/blks-core/src/simd_avx2.rs`).
  - **Coarse-Grained Subtree Slicing (256 KiB / 64 Chunks):** Reduziert Rayon-Scheduling und Heap-Allokationen um das 64-fache direkt im CPU-L2-Cache.
  - **Sequentielles Kernel-Readahead:** `MADV_SEQUENTIAL` für Mmap aktiviert $\to$ Durchsatzsprung auf **5,21 GB/s** (bzw. **6,30 GB/s** im internen Benchmark).
  - Bounds-Check-freie Deserialisierung via modernem Rust `as_chunks::<8>()` / `as_chunks::<2>()`.
  - Aggressives Inlining (`#[inline(always)]`) auf G-Funktion, Block-Kompression und Parent-Combiner; ungerollte Runden per Compile-Time-Makro.
  - Heap-Allokationen bei Base64/Hex durch Stack-Puffer eliminiert; statische `const REV_TABLE: [u8; 256]` zur Compile-Zeit.
  - C-FFI Panic-Safety: Vollständige Kapselung aller C-Entrypoints via `std::panic::catch_unwind`.
  - **ASan & UBSan bestanden:** Unter `gcc -fsanitize=address,undefined` verifiziert (0 Leaks, 0 UB).
  - **Linker- & Binary-Hardening:** Full RELRO, BIND_NOW und PIE in `.cargo/config.toml` aktiv.
  - **Constant-Time Verification (`ct_eq`):** Eliminierung von Timing-Side-Channels im Check-Modus (`-c`).
  - **SIGBUS-Schutz (`--no-mmap`):** Fallback auf sequenzielles Streaming für instabile Netzlaufwerke (NFS/CIFS).

- [x] **Kryptoanalytische Testsuite & Golden Vectors**
  - Feste Known-Good Testvektoren (`crates/blks-core/tests/test_vectors.rs`) für leere Eingaben, Blöcke, Chunks und Multi-Chunks.
  - Strict Avalanche Criterion (SAC) Verifikation (~50,1 % Bit-Diffusion bei 1-Bit Mutation).
  - Fragmentierte Stream-Feed-Tests über ungerade Puffergrenzen (1 B bis 8.192 B).
  - Thread-Invarianz-Tests über 1, 2, 4 und 8 Worker-Threads.
  - 100 % Testabdeckung mit 20 Unit-, Integrations- und C-FFI-Tests.
  - Strikte Zero-Warning-Policy unter `cargo clippy --all --tests -- -D warnings`.
  - Strikte Rustdoc-Dokumentations-Policy unter `RUSTDOCFLAGS="-D missing_docs"`.

- [x] **C-FFI Kompatibilität (`include/blks.h` & `crates/blks-core/src/ffi.rs`)**
  - C-kompatible Header-Datei `include/blks.h`.
  - C-kompatible One-Shot- und Streaming-Funktionen (`blks_hash_buffer`, `blks_hasher_new`, `blks_hasher_update`, `blks_hasher_finalize`, `blks_digest_to_base64`).
  - E2E-Validierungstest in C (`tests/test_c_ffi.c`) erfolgreich integriert.

- [x] **CLI-Werkzeug & Paketierung (`crates/blks-cli`)**
  - Multithreaded CLI `blks` mit Stdin-Pipes und Datei-Argumenten.
  - Prüfsummen-Verifikationsmodus (`-c` / `--check`).
  - Format-Auswahl (`--base64`, `--base64-url`, `--hex`, `--raw`).
  - Thread-Begrenzung (`-j` / `--threads`).
  - Strukturierte JSON-Telemetrie (`--json`).
  - Benchmark-Modus (`--benchmark`) mit Durchsatzanzeige in GB/s und integriertem 1-GiB-RAM-Benchmark ohne Argumente.
  - Terminal-Hinweis bei interaktivem Stdin.
  - Shell-Completions für Bash, Zsh, Fish und PowerShell (`completions/` und dynamisch via `--completion`).
  - Natives Arch/CachyOS `PKGBUILD` in `dist/archlinux/`.
  - Modulare Lizenzierung: `blks-core` unter **LGPL-3.0-or-later**, `blks-cli` unter **GPL-3.0-or-later**.
  - **Release v0.1.0 lokal getaggt.**

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
