#!/usr/bin/env python3
import os
import subprocess
import time
import tempfile

TOOLS = [
    ("blks (384-Bit Tree)", ["/home/meik/workspace/blks/target/release/blks"]),
    ("b3sum (BLAKE3 Tree)", ["/tmp/b3sum_bin/bin/b3sum"]),
    ("b2sum (BLAKE2b 512)", ["/usr/bin/b2sum"]),
    ("sha384sum (SHA-384)", ["/usr/bin/sha384sum"]),
    ("sha256sum (SHA-256)", ["/usr/bin/sha256sum"]),
    ("sha512sum (SHA-512)", ["/usr/bin/sha512sum"]),
    ("openssl sha384", ["/usr/bin/openssl", "dgst", "-sha384"]),
]

SIZES = [
    ("100 MB", 100 * 1024 * 1024),
    ("1 GB", 1024 * 1024 * 1024),
]

def benchmark():
    print("=" * 82)
    print(" CRYPTOGRAPHIC HASH BENCHMARK (Linux CachyOS / Zen Kernel)")
    print("=" * 82)

    for size_label, size_bytes in SIZES:
        print(f"\n--- Testgröße: {size_label} ({size_bytes / (1024*1024):.0f} MB in RAM-Cache) ---")
        
        # Create test file in /dev/shm (RAM disk) to measure pure hashing throughput without disk I/O bottleneck
        path = f"/dev/shm/blks_bench_{size_label.replace(' ', '_')}.bin"
        chunk = b"\x37" * (1024 * 1024)
        with open(path, "wb") as f:
            for _ in range(size_bytes // (1024 * 1024)):
                f.write(chunk)
        
        try:
            results = []
            for name, cmd in TOOLS:
                # Warm-up run
                subprocess.run(cmd + [path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                
                # Measure 3 runs
                runs = []
                for _ in range(3):
                    t0 = time.perf_counter()
                    subprocess.run(cmd + [path], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                    t1 = time.perf_counter()
                    runs.append(t1 - t0)
                
                best_time = min(runs)
                gbs = (size_bytes / (1024**3)) / best_time
                results.append((name, best_time * 1000.0, gbs))

            print(f"{'Tool / Algorithmus':<26} | {'Kollision':<10} | {'Output-Länge':<14} | {'Zeit (ms)':<10} | {'Durchsatz'}")
            print("-" * 82)
            
            for name, ms, gbs in sorted(results, key=lambda x: x[2], reverse=True):
                if "blks" in name:
                    coll = "192 Bit"
                    chars = "64 (Base64)"
                elif "b3sum" in name:
                    coll = "128 Bit"
                    chars = "64 (Hex)"
                elif "sha384" in name:
                    coll = "192 Bit"
                    chars = "96 (Hex)"
                elif "sha256" in name:
                    coll = "128 Bit"
                    chars = "64 (Hex)"
                elif "sha512" in name:
                    coll = "256 Bit"
                    chars = "128 (Hex)"
                elif "b2sum" in name:
                    coll = "256 Bit"
                    chars = "128 (Hex)"
                else:
                    coll = "?"
                    chars = "?"
                
                print(f"{name:<26} | {coll:<10} | {chars:<14} | {ms:8.2f} ms | {gbs:6.2f} GB/s")

        finally:
            if os.path.exists(path):
                os.remove(path)

    print("\n" + "=" * 82)

if __name__ == "__main__":
    benchmark()
