"""Shared markdown structural parser for the document-lint tools
(lint-file.py, extract-sentences.py, docjudge.py). Classifies a markdown
document into typed blocks so each tool applies the right treatment: prose
paragraphs, Enumeration/Step Block units, and headings get angloform
linting (headings and image alt text with a lighter, vocabulary-only
rule — see `docs/markdown-linting.md`); fenced code, tables, horizontal
rules, YAML frontmatter, raw HTML blocks, and footnote definitions are
excluded entirely (not prose); blockquotes and list items get their markup
stripped so their content joins the normal prose path; links resolve to
their visible text; strikethrough strips like bold/italic; reference-link
definitions drop out.

Not a full CommonMark parser — line-based, matching the rest of this
project's tooling. Extend `_LINE_KIND` before reaching for a dependency.
"""
import re
from dataclasses import dataclass, field


@dataclass
class Block:
    kind: str  # "heading" | "prose" | "enumeration" | "step_block" | "mapping" | "code" | "table" | "hr"
    text: str = ""  # de-markdowned content; for "enumeration"/"step_block", lines joined by \n
    level: int = 0  # heading level (1-6)
    raw: str = ""  # "_list" only: items with their "- " marker kept, for
    # folding into an Enumeration — the angloform grammar's own Enumeration
    # parser expects the literal bullet syntax, not de-bulleted text


_HR = re.compile(r"^\s*(-{3,}|\*{3,}|_{3,})\s*$")
_HEADING = re.compile(r"^(#{1,6})\s+(.*)$")
_FENCE = re.compile(r"^\s*```")
_TABLE_ROW = re.compile(r"^\s*\|.*\|\s*$")
_TABLE_DIVIDER = re.compile(r"^\s*\|(\s*:?-+:?\s*\|)+\s*$")
_TASK = re.compile(r"^(\s*)([-*]|\d+\.)\s+\[[ xX]\]\s+(.*)$")
_LIST_ITEM = re.compile(r"^(\s*)([-*]|\d+\.)\s+(.*)$")
_LINK_DEF = re.compile(r"^\s*\[[^\]]+\]:\s+\S+")
_FOOTNOTE_DEF = re.compile(r"^\s*\[\^[^\]]+\]:\s")
_FRONTMATTER = re.compile(r"^---\s*$")
_HTML_TAG = re.compile(r"^\s*</?[a-zA-Z][a-zA-Z0-9-]*(\s[^>]*)?/?>\s*$")


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


_IMAGE = re.compile(r"!\[([^\]]*)\]\([^)]+\)")
_FOOTNOTE_MARK = re.compile(r"\[\^[^\]]+\]")


def _inline(text: str) -> str:
    """Text-level markdown -> plain prose: links and images resolve to
    their visible text (an image inline in a sentence contributes its alt
    text the same way a link contributes its label), inline code becomes a
    angloform quoted identifier, bold/italic/strikethrough markers strip,
    a footnote marker drops (its content lives in the definition, excluded
    at the block level — see `_FOOTNOTE_DEF`)."""
    text = _FOOTNOTE_MARK.sub("", text)
    text = _IMAGE.sub(r"\1", text)  # ![alt](url) -> alt
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)  # [text](url) -> text
    text = re.sub(r"\[([^\]]+)\]\[[^\]]*\]", r"\1", text)  # [text][ref] -> text
    text = re.sub(r"`([^`]+)`", r'"\1"', text)  # inline code -> quoted
    text = re.sub(r"~~([^~]+)~~", r"\1", text)  # strikethrough
    text = re.sub(r"\*\*([^*]+)\*\*|__([^_]+)__", lambda m: m.group(1) or m.group(2), text)
    text = re.sub(r"(?<!\w)\*([^*]+)\*(?!\w)|(?<!\w)_([^_]+)_(?!\w)", lambda m: m.group(1) or m.group(2), text)
    return text


def _strip_bullet(line: str) -> str:
    m = _TASK.match(line) or _LIST_ITEM.match(line)
    return m.group(3) if m and m.lastindex == 3 else (m.group(2) if m else line.strip())


def parse_blocks(text: str) -> list[Block]:
    lines = text.split("\n")
    blocks: list[Block] = []
    i, n = 0, len(lines)
    if i == 0 and _FRONTMATTER.match(lines[0] if lines else ""):
        j = 1
        while j < n and not _FRONTMATTER.match(lines[j]):
            j += 1
        i = j + 1  # consume the closing "---" (or EOF)

    while i < n:
        raw = lines[i]
        line = raw.strip()

        if not line or _LINK_DEF.match(raw) or _FOOTNOTE_DEF.match(raw):
            i += 1
            continue

        if _HTML_TAG.match(line):
            i += 1
            while i < n and lines[i].strip():
                i += 1
            continue

        if _FENCE.match(raw):
            i += 1
            while i < n and not _FENCE.match(lines[i]):
                i += 1
            i += 1  # consume the closing fence (or EOF)
            blocks.append(Block(kind="code"))
            continue

        if line.startswith(("Date:", "Status:")):
            # a wrapped continuation (no blank line, no marker of its own)
            # belongs to this metadata line too, not the next paragraph
            i += 1
            while i < n and lines[i].strip() and not any((
                _FENCE.match(lines[i]), _HEADING.match(lines[i].strip()), _HR.match(lines[i]),
                _TABLE_ROW.match(lines[i]), lines[i].strip().startswith((">", "Date:", "Status:")),
                _TASK.match(lines[i]), _LIST_ITEM.match(lines[i]),
            )):
                i += 1
            continue

        m = _HEADING.match(line)
        if m:
            blocks.append(Block(kind="heading", text=_inline(m.group(2)).strip(), level=len(m.group(1))))
            i += 1
            continue

        if _HR.match(raw):
            blocks.append(Block(kind="hr"))
            i += 1
            continue

        if _TABLE_ROW.match(raw):
            j = i
            rows: list[str] = []
            while j < n and (_TABLE_ROW.match(lines[j]) or not lines[j].strip()):
                if lines[j].strip():
                    rows.append(lines[j].strip())
                j += 1
            # a 2-column table (header + "|---|---|" divider + data rows):
            # stash the data rows in .raw (never .text — a table stays
            # excluded from prose by default, same as every other table
            # shape) so a possible Mapping fold (ADR 0051) below can use
            # them when this table directly follows a ":"-ending statement
            data_rows = [r for r in rows[2:] if not _TABLE_DIVIDER.match(r)] if len(rows) >= 2 and _TABLE_DIVIDER.match(rows[1]) else []
            is_2col = all(r.count("|") == 3 for r in rows) if rows else False
            blocks.append(Block(kind="table", raw="\n".join(data_rows) if is_2col and data_rows else ""))
            i = j
            continue

        if line.startswith((">",)):
            quoted = []
            while i < n and lines[i].strip().startswith(">"):
                quoted.append(re.sub(r"^\s*>\s?", "", lines[i]))
                i += 1
            for b in parse_blocks("\n".join(quoted)):
                blocks.append(b)
            continue

        if line.startswith(("Given ", "When ", "Then ", "And ", "Feature:", "Scenario:")):
            block_lines = []
            while i < n and lines[i].strip().startswith(("Given ", "When ", "Then ", "And ", "Feature:", "Scenario:")):
                block_lines.append(_inline(lines[i].strip()))
                i += 1
            blocks.append(Block(kind="step_block", text="\n".join(block_lines)))
            continue

        if _TASK.match(raw) or _LIST_ITEM.match(raw):
            top_indent = _indent(raw)
            items: list[str] = []
            raw_items: list[str] = []
            j = i
            while j < n:
                l = lines[j]
                if not l.strip():
                    break
                if (_TASK.match(l) or _LIST_ITEM.match(l)) and _indent(l) == top_indent:
                    items.append(_inline(_strip_bullet(l)))
                    raw_items.append("- " + _inline(_strip_bullet(l)))
                    j += 1
                elif _indent(l) > top_indent and (_TASK.match(l) or _LIST_ITEM.match(l)):
                    # a nested sub-list under this item: kept as its own
                    # "- " lines (never concatenated into run-on prose —
                    # each sub-item is its own thought). If the parent's
                    # text ends in ":" the angloform grammar reads this as
                    # an Enumeration; if not, it correctly fails linting
                    # with an actionable reason instead of a garbled one
                    sub_indent = _indent(l)
                    while j < n and lines[j].strip() and _indent(lines[j]) == sub_indent and (_TASK.match(lines[j]) or _LIST_ITEM.match(lines[j])):
                        items[-1] += "\n- " + _inline(_strip_bullet(lines[j]))
                        raw_items[-1] += "\n- " + _inline(_strip_bullet(lines[j]))
                        j += 1
                elif _indent(l) > top_indent:
                    # a wrapped continuation of the current item's text
                    if items:
                        items[-1] = (items[-1] + " " + _inline(l.strip())).strip()
                        raw_items[-1] = (raw_items[-1] + " " + _inline(l.strip())).strip()
                    j += 1
                else:
                    break
            blocks.append(Block(kind="_list", text="\n\x00\n".join(items), raw="\n\x00\n".join(raw_items)))
            i = j
            continue

        # a plain paragraph: consume to the next blank line or block starter
        para_lines = []
        while i < n and lines[i].strip() and not any((
            _FENCE.match(lines[i]), _HEADING.match(lines[i].strip()), _HR.match(lines[i]),
            _TABLE_ROW.match(lines[i]), lines[i].strip().startswith(">"),
            _TASK.match(lines[i]), _LIST_ITEM.match(lines[i]),
            lines[i].strip().startswith(("Given ", "When ", "Then ", "And ", "Feature:", "Scenario:")),
        )):
            para_lines.append(lines[i].strip())
            i += 1
        raw_para = re.sub(r"\s+", " ", " ".join(para_lines)).strip()
        # a standalone image (a banner, a badge, a screenshot) is a title
        # fragment, not a sentence anyone wrote — same vocabulary-only rule
        # as a heading, not the full sentence grammar (docs/markdown-linting.md)
        m = re.fullmatch(_IMAGE, raw_para)
        if m:
            if m.group(1).strip():
                blocks.append(Block(kind="heading", text=_inline(m.group(1)).strip(), level=0))
            continue
        text = _inline(raw_para)
        if text.strip():
            blocks.append(Block(kind="prose", text=text))

    return _fold_enumerations(blocks)


def _fold_enumerations(blocks: list[Block]) -> list[Block]:
    """A prose block ending in ':' immediately followed by a `_list` block
    is an Enumeration (ADR 0028): the intro's last sentence plus the items
    become one unit. A `_list` with no such intro is a plain bullet list —
    still folded to one unit (each item as a line) so its content is not
    lost, but not tagged "enumeration" (no angloform Enumeration-block rule
    applies; the items are linted as an inline-list-shaped prose block)."""
    out: list[Block] = []
    i = 0
    while i < len(blocks):
        b = blocks[i]
        if (b.kind == "prose" and b.text.rstrip().endswith(":") and i + 1 < len(blocks)
                and blocks[i + 1].kind == "table" and blocks[i + 1].raw):
            # a 2-column table right after a ":"-ending statement is a
            # Mapping block (ADR 0051), same fold shape as Enumeration
            cut = b.text.rfind(". ")
            prose, intro = (b.text[: cut + 1], b.text[cut + 2 :]) if cut >= 0 else ("", b.text)
            if prose.strip():
                out.append(Block(kind="prose", text=prose.strip()))
            out.append(Block(kind="mapping", text="\n".join([intro, blocks[i + 1].raw])))
            i += 2
            continue
        if b.kind == "prose" and b.text.rstrip().endswith(":") and i + 1 < len(blocks) and blocks[i + 1].kind == "_list":
            cut = b.text.rfind(". ")
            prose, intro = (b.text[: cut + 1], b.text[cut + 2 :]) if cut >= 0 else ("", b.text)
            if prose.strip():
                out.append(Block(kind="prose", text=prose.strip()))
            raw_items = blocks[i + 1].raw.split("\n\x00\n")
            out.append(Block(kind="enumeration", text="\n".join([intro] + raw_items)))
            i += 2
            continue
        if b.kind == "_list":
            # a plain bullet list (no Enumeration intro): each item carries
            # its own sentence(s) — not one unit, unlike an Enumeration or a
            # Step Block, so each becomes its own prose block, UNLESS the
            # item itself has a nested Enumeration (its own "- " lines),
            # in which case the whole item stays one enumeration unit
            for item in b.text.split("\n\x00\n"):
                item = item.strip()
                if not item:
                    continue
                out.append(Block(kind="enumeration" if "\n" in item else "prose", text=item))
            i += 1
            continue
        out.append(b)
        i += 1
    return out
