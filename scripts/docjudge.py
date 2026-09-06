#!/usr/bin/env python3
"""Blind sub-agent judgement of whole documents: every ADR and the domain model.

Same judges as scripts/prejudge.py (docs/prejudge.md), applied to the documents
themselves rather than to repair proposals. The telephone original of an ADR is
its earliest committed version when that version predates the angloform rewrite
(ADRs written in angloform from the start have no original: naturalness and the
explainer's "unclear" flags are all we have for them).

  docjudge.py bundle OUTDIR          OUTDIR/proposals/<doc>.json (paragraphs) and
                                     OUTDIR/originals/<doc>.md (when one exists)
  docjudge.py record RESULTS.json…   merge judge outputs into docs/judgements.yaml
  docjudge.py report                 docs/judge-report.md + one summary line
  docjudge.py failing OUTDIR         one rewrite brief per failing document (JSON)

A results file is a list of {doc, naturalness?: {paragraphs: [{i, score,
issues: [{span, why}]}]}, telephone?: {explanation, unclear: [{span,
readings}], fidelity?: {score, lost, invented, distorted}}, imaginability?:
{terms: [{term, score, why}]}}. Judgements carry a hash of the document text
they were made on; a changed document shows as stale until re-judged.

Pass bar (report): naturalness mean >= 4.0 and no paragraph <= 2; fidelity >= 4
when an original exists; no unclear spans.
"""
import glob, hashlib, json, os, re, subprocess, sys, yaml

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ADRS = os.path.join(ROOT, "docs/adr")
MODEL = os.path.join(ROOT, "domain/model.json")
STORE = os.path.join(ROOT, "docs/judgements.yaml")
REPORT = os.path.join(ROOT, "docs/judge-report.md")
NAT_MEAN, NAT_MIN, FID_MIN = 4.0, 3, 4
ANGLOFORM_FROM = "0029"  # ADRs from here on were written in angloform: no English original


def sha(text):
    return hashlib.sha1(text.encode()).hexdigest()[:12]


def paragraphs(text):
    """Prose paragraphs and blocks of a markdown file; headers and metadata skipped."""
    out, cur = [], []
    for line in text.splitlines():
        if line.startswith("#") or line.startswith("Date:") or line.startswith("Status:"):
            continue
        if not line.strip():
            if cur:
                out.append("\n".join(cur))
                cur = []
            continue
        cur.append(line)
    if cur:
        out.append("\n".join(cur))
    return out


def doc_id(path):
    return os.path.basename(path).split("-")[0] if path.endswith(".md") else "model"


def original_of(path):
    if doc_id(path) >= ANGLOFORM_FROM:
        return None
    rel = os.path.relpath(path, ROOT)
    commits = subprocess.run(["git", "log", "--format=%h", "--", rel], cwd=ROOT,
                             capture_output=True, text=True).stdout.split()
    if len(commits) < 2:
        return None
    first = commits[-1]
    text = subprocess.run(["git", "show", f"{first}:{rel}"], cwd=ROOT, capture_output=True, text=True).stdout
    cur = open(path).read()
    # a real original differs from the current text beyond whitespace
    norm = lambda s: re.sub(r"\s+", " ", s).strip()
    return None if norm(text) == norm(cur) else text


def docs():
    for path in sorted(glob.glob(os.path.join(ADRS, "*.md"))):
        yield doc_id(path), path


def model_entries():
    d = json.load(open(MODEL))
    return [{"term": e["lemma"], "kind": e.get("kind", ""), "category": e["category"],
             "definition": e["definition"], "examples": e.get("examples", []),
             "member_of": e.get("member_of", "")} for e in d if e.get("definition")]


def bundle(outdir):
    os.makedirs(os.path.join(outdir, "proposals"), exist_ok=True)
    os.makedirs(os.path.join(outdir, "originals"), exist_ok=True)
    n = 0
    for did, path in docs():
        text = open(path).read()
        paras = [{"i": i + 1, "text": p} for i, p in enumerate(paragraphs(text))]
        json.dump({"doc": did, "path": os.path.relpath(path, ROOT), "hash": sha(text), "paragraphs": paras},
                  open(os.path.join(outdir, "proposals", f"{did}.json"), "w"), indent=2, ensure_ascii=False)
        orig = original_of(path)
        if orig:
            open(os.path.join(outdir, "originals", f"{did}.md"), "w").write(orig)
            n += 1
    entries = model_entries()
    json.dump({"doc": "model", "path": "domain/model.json", "hash": sha(open(MODEL).read()), "terms": entries},
              open(os.path.join(outdir, "proposals", "model.json"), "w"), indent=2, ensure_ascii=False)
    print(f"bundled {len(list(docs()))} ADRs ({n} with an English original) and {len(entries)} terms into {outdir}")


def current_hash(did):
    if did == "model":
        return sha(open(MODEL).read())
    path = next((p for d, p in docs() if d == did), None)
    return sha(open(path).read()) if path else None


def record(paths):
    store = yaml.safe_load(open(STORE)) if os.path.exists(STORE) else {}
    store = store or {}
    n = 0
    for path in paths:
        for r in json.load(open(path)):
            did = str(r["doc"])
            entry = store.setdefault(did, {})
            entry["hash"] = current_hash(did)
            for key in ("naturalness", "telephone", "imaginability"):
                if key in r:
                    if key == "telephone":  # explanation, unclear and fidelity arrive separately
                        entry.setdefault("telephone", {}).update(r[key])
                    else:
                        entry[key] = r[key]
                    n += 1
    yaml.safe_dump(store, open(STORE, "w"), allow_unicode=True, sort_keys=True, width=88)
    print(f"recorded {n} judgement(s) into docs/judgements.yaml")


def summarize(did, e):
    stale = e.get("hash") != current_hash(did)
    nat = e.get("naturalness", {}).get("paragraphs", [])
    scores = [p["score"] for p in nat if "score" in p]
    mean = sum(scores) / len(scores) if scores else None
    mn = min(scores) if scores else None
    tel = e.get("telephone", {})
    fid = tel.get("fidelity", {}).get("score")
    unclear = len(tel.get("unclear", []))
    img = e.get("imaginability", {}).get("terms", [])
    img_scores = [t["score"] for t in img if "score" in t]
    img_mean = sum(img_scores) / len(img_scores) if img_scores else None
    ok = (mean is not None and mean >= NAT_MEAN and mn is not None and mn > 2
          and (fid is None or fid >= FID_MIN) and unclear == 0 and not stale)
    return dict(stale=stale, mean=mean, min=mn, fid=fid, unclear=unclear, img=img_mean, ok=ok, n=len(scores))


def report():
    store = yaml.safe_load(open(STORE)) if os.path.exists(STORE) else {}
    rows = [(did, e, summarize(did, e)) for did, e in sorted((store or {}).items())]
    judged = [r for r in rows if r[2]["mean"] is not None]
    passing = [r for r in rows if r[2]["ok"]]
    lines = ["# Document judgement report", "",
             "Blind sub-agent judgement of every ADR and the domain model (protocol:",
             "docs/prejudge.md; tool: scripts/docjudge.py). Naturalness is the mean and",
             f"minimum paragraph score (1–5); fidelity is the telephone score against the",
             "earliest English version when one exists; unclear counts the explainer's",
             f"ambiguous spans. Pass: mean ≥ {NAT_MEAN}, no paragraph ≤ 2, fidelity ≥ {FID_MIN}, 0 unclear.", "",
             f"{len(passing)} of {len(rows)} documents pass.", "",
             "| doc | paragraphs | natural mean | natural min | fidelity | unclear | pass |", "|---|---|---|---|---|---|---|"]
    fmt = lambda v: "–" if v is None else (f"{v:.1f}" if isinstance(v, float) else str(v))
    for did, e, s in rows:
        tag = did + (" *stale*" if s["stale"] else "")
        lines.append(f"| {tag} | {s['n']} | {fmt(s['mean'])} | {fmt(s['min'])} | {fmt(s['fid'])} | {s['unclear']} | {'yes' if s['ok'] else 'no'} |")
    lines.append("")
    for did, e, s in rows:
        if s["ok"]:
            continue
        lines.append(f"## {did}")
        lines.append("")
        for p in e.get("naturalness", {}).get("paragraphs", []):
            if p.get("score", 5) <= 3 or p.get("issues"):
                lines.append(f"- ¶{p['i']} natural {p.get('score')}/5")
                for i in p.get("issues", [])[:4]:
                    lines.append(f"  - {i['span']} — {i['why']}" if isinstance(i, dict) else f"  - {i}")
        tel = e.get("telephone", {})
        for u in tel.get("unclear", []):
            lines.append(f"- unclear: {u['span']} — {u['readings']}" if isinstance(u, dict) else f"- unclear: {u}")
        f_ = tel.get("fidelity", {})
        if f_:
            lines.append(f"- fidelity {f_.get('score')}/5")
            for k in ("lost", "invented", "distorted"):
                for x in f_.get(k, []):
                    lines.append(f"  - {k}: {x}")
        for t in e.get("imaginability", {}).get("terms", []):
            if t.get("score", 5) <= 3:
                lines.append(f"- term {t['term']}: imaginable {t.get('score')}/5 — {t.get('why','')}")
        lines.append("")
    open(REPORT, "w").write("\n".join(lines))
    print(f"judge: {len(passing)}/{len(rows)} documents pass ({len(judged)} judged); report in docs/judge-report.md")


def failing(outdir):
    """Write OUTDIR/brief-<doc>.json for every judged document below the bar:
    the document's paragraphs with their scores and issues, the explainer's
    unclear spans, and the fidelity losses, so a rewriter sees exactly what to fix."""
    os.makedirs(outdir, exist_ok=True)
    store = yaml.safe_load(open(STORE)) if os.path.exists(STORE) else {}
    n = 0
    for did, e in sorted((store or {}).items()):
        s = summarize(did, e)
        if s["ok"] or s["mean"] is None or did == "model":
            continue
        path = next((p for d, p in docs() if d == did), None)
        paras = paragraphs(open(path).read())
        nat = {p["i"]: p for p in e.get("naturalness", {}).get("paragraphs", [])}
        brief = {"doc": did, "path": os.path.relpath(path, ROOT), "summary": s,
                 "paragraphs": [{"i": i + 1, "text": t, "score": nat.get(i + 1, {}).get("score"),
                                 "issues": nat.get(i + 1, {}).get("issues", [])} for i, t in enumerate(paras)],
                 "unclear": e.get("telephone", {}).get("unclear", []),
                 "fidelity": e.get("telephone", {}).get("fidelity", {})}
        json.dump(brief, open(os.path.join(outdir, f"brief-{did}.json"), "w"), indent=2, ensure_ascii=False)
        n += 1
    print(f"{n} brief(s) in {outdir}")


if __name__ == "__main__":
    a = sys.argv[1:]
    if a and a[0] == "bundle":
        bundle(a[1])
    elif a and a[0] == "record":
        record(a[1:])
    elif a and a[0] == "report":
        report()
    elif a and a[0] == "failing":
        failing(a[1])
    else:
        sys.exit(__doc__)
