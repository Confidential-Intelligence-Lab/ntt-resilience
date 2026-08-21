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

    # Historical campaign CSVs sometimes store timing values as
    # strings such as "5651541 ns". Parse the leading numeric token
    # while remaining compatible with ordinary integers/floats.
    token = text.split()[0]

    try:
        return float(token)
    except ValueError:
        return None


def median(values):
    vals = [v for v in values if v is not None and math.isfinite(v)]
    return statistics.median(vals) if vals else float("nan")


def mean(values):
    vals = [v for v in values if v is not None]
    return sum(vals) / len(vals) if vals else float("nan")


def quantile(values, q):
    vals = sorted(v for v in values if v is not None and math.isfinite(v))
    if not vals:
        return float("nan")
    if len(vals) == 1:
        return vals[0]

    pos = (len(vals) - 1) * q
    lo = math.floor(pos)
    hi = math.ceil(pos)

    if lo == hi:
        return vals[lo]

    frac = pos - lo
    return vals[lo] * (1.0 - frac) + vals[hi] * frac


def fmt(value):
    if isinstance(value, float) and math.isnan(value):
        return "nan"
    if isinstance(value, float):
        return f"{value:.6f}"
    return str(value)


def admissible_injections(df, fields):
    """
    Select campaign rows representing injections that actually executed.

    Current provenance-aware CSVs require:
      returncode == 0
      execution_valid == PASS
      fault_injections > 0

    Historical CSVs lacking these columns use the legacy fallback:
      fault_observed != ERROR
    """
    required = {"returncode", "execution_valid", "fault_injections"}

    if required.issubset(fields):
        valid = []
        for row in df:
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
        for row in df
        if row.get("fault_observed", "").strip().upper() != "ERROR"
    ]
    return valid, "legacy"


def observed(row):
    return row.get("fault_observed", "").strip().upper() == "PASS"


def detected(row):
    return row.get("detected", "").strip().lower() == "yes"


def corrected(row):
    return row.get("corrected", "").strip().lower() == "yes"


def summarize_group(rows):
    obs = [observed(r) for r in rows]
    det = [detected(r) for r in rows]
    cor = [corrected(r) for r in rows]

    obs_indices = [i for i, value in enumerate(obs) if value]
    det_indices = [i for i, value in enumerate(det) if value]

    return {
        "injections": len(rows),
        "observed_rate": mean(obs),
        "detection_rate": mean(det),
        "correction_rate": mean(cor),
        "conditional_detection_rate": (
            mean([det[i] for i in obs_indices])
            if obs_indices else float("nan")
        ),
        "correction_given_detected": (
            mean([cor[i] for i in det_indices])
            if det_indices else float("nan")
        ),
        "correction_given_observed": (
            mean([cor[i] for i in obs_indices])
            if obs_indices else float("nan")
        ),
        "median_rms": median([as_float(r.get("rms_error")) for r in rows]),
        "mean_check_failures": mean(
            [as_float(r.get("check_failures")) for r in rows]
        ),
        "median_elapsed_ns": median(
            [as_float(r.get("elapsed_ntt_ns")) for r in rows]
        ),
        "median_mitigation_ns": median(
            [as_float(r.get("mitigation_time_ns")) for r in rows]
        ),
        "median_recomputations": median(
            [as_float(r.get("recomputations")) for r in rows]
        ),
    }


def group_rows(rows, cols):
    groups = defaultdict(list)
    for row in rows:
        key = tuple(row.get(c, "") for c in cols)
        groups[key].append(row)
    return groups


def write_csv(path, rows, fields):
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)


def print_table(title, rows, fields):
    print(f"\n{title}:")
    if not rows:
        print("(no rows)")
        return

    print(",".join(fields))
    for row in rows:
        print(",".join(fmt(row.get(c, "")) for c in fields))


def summarize_campaign(path):
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

    summary_rows = []
    for key, group in sorted(group_rows(valid, group_cols).items()):
        row = dict(zip(group_cols, key))
        stats = summarize_group(group)

        for name in [
            "injections",
            "observed_rate",
            "detection_rate",
            "correction_rate",
            "median_rms",
            "mean_check_failures",
            "median_elapsed_ns",
            "median_mitigation_ns",
            "conditional_detection_rate",
        ]:
            row[name] = stats[name]

        summary_rows.append(row)

    by_site_rows = []
    site_cols = group_cols + ["fault_site"]
    for key, group in sorted(group_rows(valid, site_cols).items()):
        row = dict(zip(site_cols, key))
        stats = summarize_group(group)

        for name in [
            "injections",
            "observed_rate",
            "detection_rate",
            "correction_rate",
            "mean_check_failures",
            "median_rms",
        ]:
            row[name] = stats[name]

        by_site_rows.append(row)

    by_stage_rows = []
    stage_cols = group_cols + ["stage"]
    for key, group in sorted(group_rows(valid, stage_cols).items()):
        row = dict(zip(stage_cols, key))
        stats = summarize_group(group)

        for name in [
            "injections",
            "observed_rate",
            "detection_rate",
            "correction_rate",
            "mean_check_failures",
            "median_rms",
        ]:
            row[name] = stats[name]

        by_stage_rows.append(row)

    summary_fields = group_cols + [
        "injections",
        "observed_rate",
        "detection_rate",
        "correction_rate",
        "median_rms",
        "mean_check_failures",
        "median_elapsed_ns",
        "median_mitigation_ns",
        "conditional_detection_rate",
    ]

    by_site_fields = site_cols + [
        "injections",
        "observed_rate",
        "detection_rate",
        "correction_rate",
        "mean_check_failures",
        "median_rms",
    ]

    by_stage_fields = stage_cols + [
        "injections",
        "observed_rate",
        "detection_rate",
        "correction_rate",
        "mean_check_failures",
        "median_rms",
    ]

    print_table("Overall", summary_rows, summary_fields)
    print_table("By site", by_site_rows, by_site_fields)
    print_table("By stage", by_stage_rows, by_stage_fields)

    summary_path = path.with_suffix(".summary.csv")
    by_site_path = path.with_suffix(".by_site.csv")
    by_stage_path = path.with_suffix(".by_stage.csv")

    write_csv(summary_path, summary_rows, summary_fields)
    write_csv(by_site_path, by_site_rows, by_site_fields)
    write_csv(by_stage_path, by_stage_rows, by_stage_fields)

    print("\nWrote summary CSV files next to input.")


def summarize_overhead(path):
    with path.open(newline="") as f:
        reader = csv.DictReader(f)
        rows = list(reader)

    # Determine the baseline median for every (n, bits).
    baseline_groups = defaultdict(list)
    for row in rows:
        if (
            row.get("mitigation") == "none"
            and row.get("action") == "detect-only"
        ):
            baseline_groups[
                (row.get("n", ""), row.get("bits", ""))
            ].append(as_float(row.get("elapsed_ntt_ns")))

    baselines = {
        key: median(values)
        for key, values in baseline_groups.items()
    }

    group_cols = ["n", "bits", "mitigation", "action"]
    summary_rows = []

    for key, group in sorted(group_rows(rows, group_cols).items()):
        elapsed = [as_float(r.get("elapsed_ntt_ns")) for r in group]
        mitigation_ns = [
            as_float(r.get("mitigation_time_ns")) for r in group
        ]
        checks = [as_float(r.get("checks_performed")) for r in group]
        stage_checks = [as_float(r.get("stage_checks")) for r in group]
        recomputations = [
            as_float(r.get("recomputations")) for r in group
        ]

        baseline = baselines.get((key[0], key[1]), float("nan"))
        med_elapsed = median(elapsed)

        if (
            baseline is not None
            and not math.isnan(baseline)
            and baseline != 0
            and not math.isnan(med_elapsed)
        ):
            overhead = 100.0 * (med_elapsed - baseline) / baseline
        else:
            overhead = float("nan")

        row = dict(zip(group_cols, key))
        row.update({
            "runs": len(group),
            "median_elapsed_ns": med_elapsed,
            "p25_elapsed_ns": quantile(elapsed, 0.25),
            "p75_elapsed_ns": quantile(elapsed, 0.75),
            "median_mitigation_ns": median(mitigation_ns),
            "median_checks": median(checks),
            "median_stage_checks": median(stage_checks),
            "median_recomputations": median(recomputations),
            "baseline_elapsed_ns": baseline,
            "overhead_pct_vs_none": overhead,
        })

        summary_rows.append(row)

    fields = group_cols + [
        "runs",
        "median_elapsed_ns",
        "p25_elapsed_ns",
        "p75_elapsed_ns",
        "median_mitigation_ns",
        "median_checks",
        "median_stage_checks",
        "median_recomputations",
        "baseline_elapsed_ns",
        "overhead_pct_vs_none",
    ]

    print_table("Overhead", summary_rows, fields)

    out = path.with_suffix(".summary.csv")
    write_csv(out, summary_rows, fields)
    print("\nWrote", out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("csv", type=Path)
    ap.add_argument(
        "--kind",
        choices=["campaign", "overhead"],
        default="campaign",
    )
    args = ap.parse_args()

    if args.kind == "campaign":
        summarize_campaign(args.csv)
    else:
        summarize_overhead(args.csv)


if __name__ == "__main__":
    main()
