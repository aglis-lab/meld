import json
import glob
import csv
import os
from collections import defaultdict

# group -> list of row dicts
groups = defaultdict(list)

estimate_paths = glob.glob("target/criterion/**/new/estimates.json", recursive=True)
print("Files found:", len(estimate_paths))

for est_path in estimate_paths:
    dir_path = os.path.dirname(est_path)  # .../<group>/<param>/new
    bench_path = os.path.join(dir_path, "benchmark.json")

    if not os.path.exists(bench_path):
        print(f"Skipping {est_path}: no benchmark.json found")
        continue

    with open(bench_path) as f:
        bench = json.load(f)

    group = bench["group_id"]

    throughput_info = bench.get("throughput")
    if not throughput_info or "Elements" not in throughput_info:
        print(f"Skipping {bench_path}: no Elements throughput recorded")
        continue

    n = throughput_info["Elements"]  # real element/iteration count, straight from Criterion

    with open(est_path) as f:
        est = json.load(f)

    mean_ns = est["mean"]["point_estimate"]
    std_ns  = est["std_dev"]["point_estimate"]

    groups[group].append({
        "n":              n,
        "duration_ns":    round(mean_ns, 3),          # total bench duration, raw ns
        "std_ns":         round(std_ns / n, 3),        # per-op std dev, raw ns
        "throughput":     round(n / (mean_ns / 1e9), 3),  # raw elements/sec (ops/sec)
    })

os.makedirs("doc/stats", exist_ok=True)

fieldnames = ["n", "duration_ns", "std_ns", "throughput"]
print(f"Writing {len(groups)} groups to doc/stats/*.csv")
for group, rows in groups.items():
    rows.sort(key=lambda r: r["n"])

    # sanitize group name for use as filename
    safe_name = group.replace("/", "_").replace(" ", "_")
    out_path = f"doc/stats/{safe_name}.csv"

    with open(out_path, "w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)

    print(f"[{group}] → {out_path} ({len(rows)} rows)")
