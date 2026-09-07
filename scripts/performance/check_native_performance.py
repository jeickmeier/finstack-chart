#!/usr/bin/env python3
"""Join actual MTLDrawable presentation callbacks to native chart paint stamps.

No callback timestamp, CPU paint acknowledgement or skipped drawable is substituted
for presentedTime. Host/Unix clock correspondence is captured for each drawable.
"""
import argparse
import bisect
import gzip
import json
import math
from pathlib import Path


def read(path):
    p = Path(path)
    content = gzip.decompress(p.read_bytes()).decode() if p.suffix == ".gz" else p.read_text()
    return [json.loads(line) for line in content.splitlines() if line.strip()]


def quantiles(values):
    values = sorted(values)
    return {"count": len(values), **{name: values[max(0, math.ceil(p * len(values)) - 1)] if values else None for name, p in [("p50", .5), ("p95", .95), ("p99", .99), ("max", 1)]}}


def frames(events, metal):
    painted = [e for e in events if e["event"] == "scene-frame-painted"]
    starts = [int(e["unix_ns"]) for e in events if e["event"] == "scene-frame-begin"]
    ends = [int(e["unix_ns"]) for e in painted]
    presented = {(e["layer"], e["drawable"]): e for e in metal if e["event"] == "metal-presented"}
    gpu = {}
    for e in metal:
        if e["event"] == "metal-completed":
            gpu.setdefault((e["layer"], e["drawable"]), []).append(e)
    result = []
    missing = skipped = 0
    for d in (e for e in metal if e["event"] == "metal-drawable"):
        key = d["layer"], d["drawable"]
        p = presented.get(key)
        if p is None:
            missing += 1
            continue
        if int(p["presented_host_ns"]) == 0:
            skipped += 1
            continue
        unix = int(d["unix_ns"])
        index = bisect.bisect_right(ends, unix) - 1
        assert index >= 0, "displayed drawable has no chart paint stamp"
        stamp = painted[index]
        start_index = bisect.bisect_right(starts, ends[index]) - 1
        assert start_index >= 0
        display = unix + int(p["presented_host_ns"]) - int(d["host_ns"])
        assert display >= ends[index], "display precedes attributed chart paint"
        completions = gpu.get(key, [])
        assert completions, "missing GPU completion for displayed drawable"
        assert all(e["status"] == 4 and 0 < int(e["gpu_start_ns"]) <= int(e["gpu_end_ns"]) for e in completions)
        gpu_start = min(int(e["gpu_start_ns"]) for e in completions)
        gpu_end = max(int(e["gpu_end_ns"]) for e in completions)
        result.append({"layer": key[0], "drawable": key[1], "paint_unix_ns": str(ends[index]), "display_unix_ns": str(display), "scene_begin_to_display_ns": display - starts[start_index], "paint_to_display_ns": display - ends[index], "gpu_ns": gpu_end - gpu_start, "drawable_unix_ns":str(unix), "gpu_start_unix_ns":str(unix+gpu_start-int(d["host_ns"])), "gpu_end_unix_ns":str(unix+gpu_end-int(d["host_ns"])), "charts": stamp["charts"]})
    result.sort(key=lambda e: int(e["display_unix_ns"]))
    return result, {"displayed": len(result), "skipped": skipped, "missing_callbacks": missing}


def stream(events, display, required_seconds, partial=False):
    protocol = next(e for e in events if e["event"] == "protocol")
    assert protocol["seconds"] == required_seconds
    commits = [e for e in events if e["event"] == "commit"]
    expected = required_seconds * 10 + protocol["warmup_batches"]
    assert 0 < len(commits) < expected if partial else len(commits) == expected
    if partial:
        expected = len(commits)
    assert [e["tick"] for e in commits] == list(range(1, expected + 1))
    assert [e["revision"] for e in commits] == list(range(2, expected + 2))
    assert all(e["counts"][0]["inserted"] == 999 and e["counts"][0]["updated"] == 1 for e in commits)
    assert all(sum(c["removed"] for c in e["counts"]) == (1 if e["tick"] % 10 == 0 else 0) for e in commits)
    assert all(e["retained"] == (99999 if e["tick"] % 10 == 0 else 100000) for e in commits)
    # First actual displayed scene whose coherent source includes each committed operation.
    by_revision = {}
    for f in display:
        if f["charts"] and f["charts"][0]["stamp"]:
            revision = int(f["charts"][0]["stamp"]["store"])
            by_revision.setdefault(revision, int(f["display_unix_ns"]))
    revisions = sorted(by_revision)
    lags = []
    measured = [e for e in commits if e["measured"]]
    undisplayed = sum(c["revision"] > revisions[-1] for c in measured)
    if partial:
        measured = [c for c in measured if c["revision"] <= revisions[-1]]
    for c in measured:
        i = bisect.bisect_left(revisions, c["revision"])
        assert i < len(revisions), f"committed revision {c['revision']} never displayed"
        arrival = int(c.get("arrival_unix_ns", int(protocol["unix_ns"]) + int(c["arrival_ns"])))
        lag = by_revision[revisions[i]] - arrival
        assert lag >= 0
        lags.append(lag)
    disposed = next((e for e in events if e["event"] == "disposed"), None)
    if not partial:
        assert disposed is not None
        assert not disposed["chart_live"] and disposed["tracked_export_scenes_live"] == 0
        assert all(disposed["export_metrics"][n] == 0 for n in ["pending", "running", "rows", "input_bytes"])
        complete = next(e for e in events if e["event"] == "complete")
        assert complete["pending_ack"] == 0
    captures = {e["id"]: e for e in events if e["event"] == "export-capture"}
    outputs = {e["id"]: e for e in events if e["event"] == "export-complete"}
    assert captures.keys() == outputs.keys() and captures
    for key, capture in captures.items():
        # Output manifests add preparation statistics; immutable capture identity must agree.
        source = capture["manifest"]
        out = outputs[key]["manifest"]
        for field in ["origin_scene", "definition", "source_epoch", "state", "fonts", "profile", "interaction_policy"]:
            if field in source:
                assert source[field] == out[field], (key, field)
        assert source["store"] == out["stamp"]["store"]
        assert [(d["version"], d["schema"]["version"]) for d in source["datasets"]] == [tuple(d) for d in out["datasets"]]
    overlap, baseline = [], []
    intervals = [(int(captures[k]["ns"]), int(outputs[k]["ns"])) for k in captures]
    for c, lag_ns in zip(measured, lags):
        (overlap if any(a <= int(c["arrival_ns"]) <= b for a, b in intervals) else baseline).append(lag_ns)
    export_comparison = {"overlap_ingest_to_present_ns": quantiles(overlap), "other_ingest_to_present_ns": quantiles(baseline), "worker_ns": quantiles([int(e["worker_ns"]) for e in outputs.values()]), "interpretation": "Descriptive overlap samples; no causal overhead estimate from a small sample"}
    lag = quantiles(lags)
    commit_ms = quantiles([int(e["commit_ns"])/1e6 for e in measured])
    hover_ms = quantiles([(int(e["query_ns"]) + int(e["dispatch_ns"]))/1e6 for e in events if e["event"] == "hover" and e["tick"] > protocol["warmup_batches"] and e["tick"] <= expected])
    return {"seconds": required_seconds, "full_sustained_protocol": not partial and required_seconds >= 1800, "owner_stopped_partial": partial, "unobserved_tail_commits": undisplayed, "accepted_commits": len(commits), "observed_seconds_after_warmup": (len(commits)-protocol["warmup_batches"])/10, "measured_commits": len(measured), "measured_upsert_rows": len(measured) * 1000, "ingest_to_present_ns": lag, "ingest_to_present_budget_pass": lag["p95"] <= 250_000_000, "commit_ms": commit_ms, "hover_cpu_ms": hover_ms, "export_count": len(captures), "export_comparison": export_comparison, "peak_sampled_rss_kib": max((e.get("rss_kib") or 0) for e in events), "accepted_operations_reconciled": True, "chart_and_export_scenes_released": disposed is not None and not disposed["chart_live"], "rss_kib": [e["rss_kib"] for e in events if e["event"] == "reference-check"]}


def finite(events, display):
    protocol=next(e for e in events if e["event"]=="finite-protocol")
    draws=[e for e in events if e["event"]=="gpui-draw"]
    submits=[e for e in events if e["event"]=="gpui-submit"]
    draw_starts=[int(e["start_unix_ns"]) for e in draws]
    submit_starts=[int(e["start_unix_ns"]) for e in submits]
    samples=[]
    for f in display:
        i=bisect.bisect_right(draw_starts,int(f["paint_unix_ns"]))-1
        j=bisect.bisect_right(submit_starts,int(f["drawable_unix_ns"]))-1
        assert i>=0 and j>=0
        d,t=draws[i],submits[j]
        if not (protocol["warmup_ticks"]<d["tick"]<=protocol["warmup_ticks"]+protocol["measured_ticks"]) or not f["charts"]:
            continue
        # Conservative sum: any CPU/GPU overlap is counted twice. Queue/display latency is
        # retained independently; it is not confused with a pipeline's per-frame work.
        work=int(d["draw_ns"])+int(t["submit_ns"])+f["gpu_ns"]
        samples.append({"drawable":f["drawable"],"draw_ns":int(d["draw_ns"]),"submit_ns":int(t["submit_ns"]),"gpu_ns":f["gpu_ns"],"total_frame_work_ns":work,"draw_to_gpu_complete_ns":int(f["gpu_end_unix_ns"])-int(d["start_unix_ns"]),"draw_to_display_ns":int(f["display_unix_ns"])-int(d["start_unix_ns"]),"display_unix_ns":f["display_unix_ns"]})
    assert len(samples)>=30
    h=[e for e in events if e["event"]=="finite-hover" and e["measured"]]
    hover=quantiles([int(e["query_ns"])+int(e["dispatch_ns"]) for e in h])
    work=quantiles([e["total_frame_work_ns"] for e in samples])
    final=next(e for e in events if e["event"]=="finite-disposed")
    assert final["live_entities"]==0
    snapshots=[e for e in events if e["event"] in ["finite-sample", "finite-drained"] and e["charts"]]
    layouts=[x["charts"][0]["layouts"] for x in snapshots]
    if protocol["mode"] in ["lines","million"]:
        assert len(set(layouts))==1,"hover rebuilt full chart layout/index"
        assert all(x["charts"][0]["bounds"]["width"]==1200 for x in snapshots)
        assert max(x["examined"] for x in h)<=40 if protocol["mode"]=="lines" else max(x["examined"] for x in h)<=4
    if protocol["mode"].startswith("dashboard"):
        assert len(snapshots[-1]["charts"])==13
        assert snapshots[-1]["charts"][0]["density"]["source_rows"]==50000
        for c in snapshots[-1]["charts"]:
            assert int(c["schedule"]["committed"])==120 and int(c["schedule"]["presented"])==120
            assert c["schedule"]["active"]==0 and c["schedule"]["pending"]==0
    return {"mode":protocol["mode"],"hover_cpu_ns":hover,"total_frame_work_ns":work,"frame_work_budget_pass":work["p95"]<=16_700_000,"hover_budget_pass":hover["p95"]<4_000_000,"draw_to_gpu_complete_ns":quantiles([e["draw_to_gpu_complete_ns"] for e in samples]),"draw_to_display_ns":quantiles([e["draw_to_display_ns"] for e in samples]),"display_interval_ns":quantiles([int(b["display_unix_ns"])-int(a["display_unix_ns"]) for a,b in zip(samples,samples[1:])]),"full_frame_samples":samples,"snapshots":snapshots,"disposed":final}

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("events")
    parser.add_argument("metal")
    parser.add_argument("--seconds", type=int)
    parser.add_argument("--partial-stream", action="store_true", help="Report an interrupted prefix without claiming full duration or graceful disposal")
    parser.add_argument("--frames", required=True)
    args = parser.parse_args()
    events, metal = read(args.events), read(args.metal)
    display, accounting = frames(events, metal)
    with Path(args.frames).open("w") as out:
        for row in display:
            out.write(json.dumps(row, separators=(",", ":")) + "\n")
    report = {"callbacks": accounting, "gpu_ns": quantiles([f["gpu_ns"] for f in display]), "scene_begin_to_display_ns": quantiles([f["scene_begin_to_display_ns"] for f in display]), "workload": stream(events, display, args.seconds, args.partial_stream) if args.seconds is not None else finite(events,display)}
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
