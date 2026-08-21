#!/usr/bin/env python3

import argparse
import csv
import math
import statistics
from collections import defaultdict
from pathlib import Path


def as_float(value):
    try:
        return float(value)
    except (TypeError, ValueError):
        return None


def median(values):
    vals = [v for v in values if v is not None and math.isfinite(v)]
    return statistics.median(vals) if vals else float("nan")


def mean(values):
    vals = [v for v in values if v is not None]
    return sum(vals) / len(vals) if vals else float("nan")


def fmt(value):
    if isinstance(value, float) and math.isnan(value):
        return "nan"
    if isinstance(value, float):
        return f"{value:.6f}"
    return str(value)


def admissible(row, has_provenance):
    if has_provenance:
        try:
            return (
                int(row["returncode"]) == 0
                and row["execution_valid"].strip().upper() == "PASS"
                and int(row["fault_injections"]) > 0
            )
        except (KeyError, TypeError, ValueError):
            return False

    # Backward compatibility for historical campaign CSVs.
    return row.get("fault_observed", "").strip().upper() != "ERROR"


def summarize_group(rows):
    observed = [
        r.get("fault_observed", "").strip().upper() == "PASS"
        for r in rows
    ]
    detected = [
        r.get("detected", "").strip().lower() == "yes"
        for r in rows
    ]
    corrected = [
        r.get("corrected", "").strip().lower() == "yes"
        for r in rows
    ]

    obs_indices = [i for i, value in enumerate(observed) if value]
    det_indices = [i for i, value in enumerate(detected) if value]

    return {
        "injections": len(rows),
        "observability": mean(observed),
        "detection_all": mean(detected),
        "correction_all": mean(corrected),
        "median_rms": median([as_float(r.get("rms_error")) for r in rows]),
        "mean_check_failures": mean(
            [as_float(r.get("check_failures")) for r in rows]
        ),
        "median_recomputations": median(
            [as_float(r.get("recomputations")) for r in rows]
        ),
        "median_elapsed_ns": median(
            [as_float(r.get("elapsed_ntt_ns")) for r in rows]
        ),
        "detection_given_observed": (
            mean([detected[i] for i in obs_indices])
            if obs_indices else float("nan")
        ),
        "correction_given_detected": (
            mean([corrected[i] for i in det_indices])
            if det_indices else float("nan")
        ),
        "correction_given_observed": (
            mean([corrected[i] for i in obs_indices])
            if obs_indices else float("nan")
        ),
    }


def write_csv(path, rows, fields):
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def main(path):
    with path.open(newline="") as f:
        reader = csv.DictReader(f)
        rows = list(reader)
        fields = set(reader.fieldnames or [])

    required = {"returncode", "execution_valid", "fault_injections"}
    has_provenance = required.issubset(fields)

    valid = [r for r in rows if admissible(r, has_provenance)]

    print(
        f"Rows: {len(rows)}  "
        f"Admissible injections: {len(valid)}  "
        f"Excluded: {len(rows) - len(valid)}"
    )

    gcols = ["mode", "n", "bits", "mitigation", "action"]

    groups = defaultdict(list)
    site_groups = defaultdict(list)

    for row in valid:
        key = tuple(row.get(c, "") for c in gcols)
        groups[key].append(row)

        site_key = key + (row.get("fault_site", ""),)
        site_groups[site_key].append(row)

    summary_rows = []
    for key in sorted(groups):
        result = dict(zip(gcols, key))
        result.update(summarize_group(groups[key]))
        summary_rows.append(result)

    by_site_rows = []
    for key in sorted(site_groups):
        result = dict(zip(gcols + ["fault_site"], key))
        stats = summarize_group(site_groups[key])

        for name in [
            "injections",
            "observability",
            "detection_all",
            "correction_all",
            "mean_check_failures",
            "median_rms",
        ]:
            result[name] = stats[name]

        by_site_rows.append(result)

    summary_fields = gcols + [
        "injections",
        "observability",
        "detection_all",
        "correction_all",
        "median_rms",
        "mean_check_failures",
        "median_recomputations",
        "median_elapsed_ns",
        "detection_given_observed",
        "correction_given_detected",
        "correction_given_observed",
    ]

    site_fields = gcols + [
        "fault_site",
        "injections",
        "observability",
        "detection_all",
        "correction_all",
        "mean_check_failures",
        "median_rms",
    ]

    prefix = str(path.with_suffix(""))
    summary_path = Path(prefix + ".summary.csv")
    site_path = Path(prefix + ".by_site.csv")

    write_csv(summary_path, summary_rows, summary_fields)
    write_csv(site_path, by_site_rows, site_fields)

    if summary_rows:
        print()
        print(",".join(summary_fields))
        for row in summary_rows:
            print(",".join(fmt(row.get(c, "")) for c in summary_fields))

    print("Wrote", summary_path, site_path)


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("csv", type=Path)
    main(ap.parse_args().csv)
