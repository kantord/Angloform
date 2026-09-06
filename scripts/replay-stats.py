#!/usr/bin/env python3
"""No-LLM proxy for the paragraph repair flow (ADR 0012 review still human).

Re-validates every stored proposal in tests/paragraph-cases/ against the
current linter (via `agenttest paragraphs --dry-run`, which rewrites the
cases) and reports: valid proposals per paragraph and overall, rejections
with no actionable advice, and the blocking words ranked. Seconds, no API.
"""
import glob, os, re, subprocess, sys, collections, yaml

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CASES = os.path.join(ROOT, "tests/paragraph-cases")

def main():
    sources = sorted({yaml.safe_load(open(f))["source"] for f in glob.glob(os.path.join(CASES, "*.yaml"))})
    for src in sources:
        subprocess.run(["cargo", "run", "-q", "-p", "agenttest", "--", "paragraphs", src,
                        os.path.join(ROOT, "docs/paragraph-report.md"), "--dry-run"],
                       cwd=ROOT, check=True, capture_output=True)
    total = valid = noadvice = 0
    words = collections.Counter(); findings = collections.Counter()
    rows = []
    for f in sorted(glob.glob(os.path.join(CASES, "*.yaml"))):
        d = yaml.safe_load(open(f))
        ps = d.get("proposals", [])
        v = sum(1 for p in ps if p["valid"])
        total += len(ps); valid += v
        rows.append((d["index"], os.path.basename(d["source"]), v, len(ps), bool(d.get("best")), d.get("verdict")))
        for p in ps:
            if p["valid"]:
                continue
            diag = p.get("diagnosis") or ""
            if "not recognizable" in diag:
                noadvice += 1
            for m in re.finditer(r'WORD: "([^"]+)" is (?:not an angloform word|banned in angloform)', diag):
                words[m.group(1).lower()] += 1
            for m in re.finditer(r'\[all unknown words: ([^\]]+)\]', diag):
                for w in m.group(1).split(", "):
                    words[w] += 1
            for m in re.finditer(r'STYLE: ([^|]*?)(?: —|$)', diag):
                findings[m.group(1).strip()[:60]] += 1
    print(f"replay: {valid}/{total} stored proposals valid under the current linter; "
          f"{noadvice} rejections with no actionable advice")
    for i, src, v, n, best, verdict in rows:
        print(f"  [{i}] {src}: {v}/{n} valid · best {'yes' if best else 'no '} · verdict {verdict}")
    print("blocking words:", ", ".join(f"{w} ×{n}" for w, n in words.most_common(12)))
    print("findings:", ", ".join(f"{k} ×{n}" for k, n in findings.most_common(6)))

if __name__ == "__main__":
    main()
