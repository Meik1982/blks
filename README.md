# blks – High-Performance 384-Bit Cryptographic Tree-Hash

`blks` (**Blk**-**S**um) ist ein modernes, multithreaded kryptografisches Baum-Prüfsummenwerkzeug, das speziell entwickelt wurde, um die fundamentale 128-Bit-Kollisionsbarriere von BLAKE3 und SHA-256 zu durchbrechen – ohne dabei auf die vertraute 64-Zeichen-Zeilenbreite im Terminal zu verzichten.

---

## 🎯 Motivation & Mathematische Eleganz

### 1. Die 64-Zeichen-Symmetrie (Base64 ohne Padding)
* **SHA-256 (Hex):** 256 Bit = 32 Byte = **64 Zeichen** (Hexadezimal: 4 Bit pro Zeichen, $64 \times 4 = 256$ Bit).
* **blks-384 (Base64):** 384 Bit = 48 Byte = **exakt 64 Zeichen** (Base64: 6 Bit pro Zeichen, $48 / 3 \times 4 = 64$ Zeichen).
* **Vorteil:** Ein `blks`-Hash belegt in Shell-Pipelines, Checksum-Dateien (`.blks`), Logfiles und JSON-Strukturen **haargenau dieselbe Zeichenbreite wie ein klassischer SHA-256-Hash**, transportiert aber **50 % mehr kryptografische Entropie** – völlig ohne unschönes Padding (`=`).

```text
SHA-256 (Hex):   03a4609994f8fb78719a21a20584713844cf9d0161a64e371e5968f79382a833  (64 Chars)
blks-384 (B64):  fLm4TI9xjaLm7jjLJioInotHz8FrvDh09LZBoyhM3Ogg22uDK84CJDxgCB9D0cE+  (64 Chars)
```

### 2. Durchbrechen der 128-Bit-Kollisionsbarriere
* **BLAKE3 & SHA-256 Flaschenhals:** BLAKE3 verwendet intern einen Chaining Value (CV) von 256 Bit. Nach dem Geburtstagsparadoxon liegt die theoretische Obergrenze der Kollisionsresistenz unverrückbar bei $256 / 2 = \mathbf{128\text{ Bit}}$. Jede künstliche XOF-Streckung auf 512 Bit erzeugt nur Scheinsicherheit, da ein Angreifer den internen 256-Bit-Zustand angreift.
* **blks-384:** Arbeitet durchgängig mit einem 512-Bit Zustand (8 × 64-Bit Wörter) und 384-Bit (48-Byte) Chaining Values im gesamten Merkle-Baum.
  * **Kollisionsresistenz:** $\ge \mathbf{192\text{ Bit}}$ ($2^{192}$ Operationen – um den Faktor $2^{64} \approx 1,84 \times 10^{19}$ schwerer zu brechen als BLAKE3/SHA-256).
  * **Preimage-Resistenz:** $2^{384}$ Operationen.
  * **Post-Quantum:** Vollständig resistent gegen Grover-Quantenalgorithmen ($\ge 192$ Bit Sicherheit).

---

## ⚡ Kernarchitektur

1. **64-Bit ARX-Kompressionsfunktion:**
   * Basiert auf der 64-Bit BLAKE2b-Permutation mit vollen 12 kryptografischen Runden.
   * Verarbeitet 128-Byte-Nachrichtenblöcke.
   * Eigene Domain-Separation und Initialisierungskonstanten (`BLKS_384`).
2. **Merkle-Baum & Linux Page-Alignment:**
   * Leaf-Chunks sind auf **4096 Bytes (4 KiB)** dimensioniert (perfekt abgestimmt auf Linux Page-Cache, NVMe-Sektoren und Direct-I/O-Blöcke).
   * Parallele Leaf-Berechnung via `rayon` Work-Stealing über alle CPU-Kerne.
   * Baumreduktion: Zwei 48-Byte Child-Hashes (zusammen 96 Bytes) passen ohne Überlauf in einen einzigen 128-Byte-Parent-Block – exakt ein Kompressionsschritt pro Elternknoten!
3. **Zero-Copy I/O:**
   * Automatische Speicherabbildung regulärer Dateien (`memmap2`) mit Durchsätzen über 2 GB/s.
   * Deterministischer Streaming-Modus für Pipes (`stdin`) mit bit-exakter Parität zum parallelen Slice-Pfad.
4. **C-FFI Schnittstelle (`include/blks.h`):**
   * Exportiert `libblks_core.a` / `libblks_core.so` für direkte Anbindung in C-Tools wie `blkcp`.

---

## 🚀 Installation & Verwendung

### Kompilieren & Testen
```bash
make test
```

### CLI-Aufruf
```bash
# Datei hashen (Standard 64-Zeichen Base64)
blks datei.iso

# Stdin hashen
cat daten.bin | blks

# Prüfsummenliste erstellen
blks datei1 datei2 > checksums.blks

# Prüfsummen verifizieren
blks -c checksums.blks

# Benchmark-Modus mit Durchsatzanzeige
blks --benchmark datei.iso

# Strukturierte JSON-Telemetrie
blks --json datei.iso
```

---

## 📁 Workspace-Struktur
* `crates/blks-core`: Krypto-Engine, Merkle-Baum, Base64-Kodierer und C-FFI.
* `crates/blks-cli`: CLI-Werkzeug `blks`.
* `include/blks.h`: C-Header für die FFI-Integration.
* `tests/`: Integrationstests und C-FFI-Verifikationstest.
