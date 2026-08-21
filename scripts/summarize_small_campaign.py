#!/usr/bin/env python3

import argparse
import csv
import math
import statistics
from collections import defaultdict
from pathlib import Path


def as_float(value):
    if value is None:
        return None

    text = str(value).strip()

    if not text or text.upper() in {"NA", "N/A", "ERROR"}:
        return None

    if text.lower() in {"inf", "+inf", "infinity", "+infinity"}:
        return float("inf")

    if text.lower() in {"-inf", "-infinity"}:
        return float("-inf")

    token = text.split()[0]

    try:
        return float(token)
    except ValueError:
        return None


def mean(values):
    vals = [v for v in values if v is not None]
    return sum(vals) / len(vals) if vals else float("nan")


def median(values):
    vals = [v for v in values if v is not None and math.isfinite(v)]
    return statistics.median(vals) if vals else float("nan")


def fmt(value):
    if isinstance(value, float) and math.isnan(value):
        return "nan"
    if isinstance(value, float):
        return f"{value:.6f}"
    return str(value)


def pass_bool(value):
    return str(value).strip().upper() == "PASS"


def yes_bool(value):
    return str(value).strip().lower() == "yes"


def admissible_injections(rows, fields):
    """
    Current campaign CSVs:
      returncode == 0
      execution_valid == PASS
      fault_injections > 0

    Historical CSVs lacking these fields use the legacy fallback:
      fault_observed != ERROR
    """
    required = {"returncode", "execution_valid", "fault_injections"}

    if required.issubset(fields):
        valid = []

        for row in rows:
            try:
                ok = (
                    int(row.get("returncode", "")) == 0
                    and row.get("execution_valid", "").strip().upper() == "PASS"
                    and int(row.get("fault_injections", "0")) > 0
                )
            except (TypeError, ValueError):
                ok = False

            if ok:
                valid.append(row)

        return valid, "provenance"

    valid = [
        row
        for row in rows
        if row.get("fault_observed", "").strip().upper() != "ERROR"
    ]

    return valid, "legacy"


def group_rows(rows, cols):
    groups = defaultdict(list)

    for row in rows:
        key = tuple(row.get(c, "") for c in cols)
        groups[key].append(row)

    return groups


def summarize_group(rows):
    observed = [pass_bool(r.get("fault_observed", "")) for r in rows]
    detected = [yes_bool(r.get("detected", "")) for r in rows]
    corrected = [yes_bool(r.get("corrected", "")) for r in rows]

    obs_indices = [i for i, value in enumerate(observed) if value]
    det_indices = [i for i, value in enumerate(detected) if value]

    return {
        "injections": len(rows),
        "observed_rate": mean(observed),
        "detection_rate_all": mean(detected),
        "correction_rate_all": mean(corrected),
        "detection_rate_observed": (
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
        "median_rms": median(
            [as_float(r.get("rms_error")) for r in rows]
        ),
        "mean_check_failures": mean(
            [as_float(r.get("check_failures")) for r in rows]
        ),
        "median_recomputations": median(
            [as_float(r.get("recomputations")) for r in rows]
        ),
        "median_elapsed_ns": median(
            [as_float(r.get("elapsed_ntt_ns")) for r in rows]
        ),
        "median_mitigation_ns": median(
            [as_float(r.get("mitigation_time_ns")) for r in rows]
        ),
    }


def write_csv(path, rows, fields):
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def print_table(name, rows, fields):
    print(f"\n=== {name} ===")

    if not rows:
        print("(no rows)")
        return

    print(",".join(fields))

    for row in rows:
        print(",".join(fmt(row.get(c, "")) for c in fields))


def main(path):
    with path.open(newline="") as f:
        reader = csv.DictReader(f)
        rows = list(reader)
        fields = set(reader.fieldnames or [])

    valid, mode = admissible_injections(rows, fields)

    print(
        f"Rows: {len(rows)}  "
        f"Admissible injections: {len(valid)}  "
        f"Excluded: {len(rows) - len(valid)}  "
        f"Filter: {mode}"
    )

    group_cols = ["n", "bits", "mitigation", "action"]

    overall_rows = []

    for key, group in sorted(group_rows(valid, group_cols).items()):
        row = dict(zip(group_cols, key))
        stats = summarize_group(group)

        for name in [
            "injections",
            "observed_rate",
            "detection_rate_all",
            "correction_rate_all",
            "median_rms",
            "mean_check_failures",
            "median_recomputations",
            "median_elapsed_ns",
            "median_mitigation_ns",
            "detection_rate_observed",
            "correction_given_detected",
            "correction_given_observed",
        ]:
            row[name] = stats[name]

        overall_rows.append(row)

    overall_fields = group_cols + [
        "injections",
        "observed_rate",
        "detection_rate_all",
        "correction_rate_all",
        "median_rms",
        "mean_check_failures",
        "median_recomputations",
        "median_elapsed_ns",
        "median_mitigation_ns",
        "detection_rate_observed",
        "correction_given_detected",
        "correction_given_observed",
    ]

    outputs = {
        "summary": (overall_rows, overall_fields),
    }

    breakdowns = {
        "by_site": group_cols + ["fault_site"],
        "by_stage": group_cols + ["stage"],
        "by_bit": group_cols + ["bit"],
        "by_op": group_cols + ["fault_op"],
    }

    for name, cols in breakdowns.items():
        table = []

        for key, group in sorted(group_rows(valid, cols).items()):
            row = dict(zip(cols, key))
            stats = summarize_group(group)

            for stat_name in [
                "injections",
                "observed_rate",
                "detection_rate_all",
                "correction_rate_all",
                "mean_check_failures",
                "median_rms",
            ]:
                row[stat_name] = stats[stat_name]

            table.append(row)

        table_fields = cols + [
            "injections",
            "observed_rate",
            "detection_rate_all",
            "correction_rate_all",
            "mean_check_failures",
            "median_rms",
        ]

        outputs[name] = (table, table_fields)

    prefix = str(path.with_suffix(""))

    for name, (table, table_fields) in outputs.items():
        print_table(name, table, table_fields)

        write_csv(
            Path(prefix + f".{name}.csv"),
            table,
            table_fields,
        )

    print("\nWrote summary CSV files next to input.")


if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("csv", type=Path)
    args = ap.parse_args()
    main(args.csv)
