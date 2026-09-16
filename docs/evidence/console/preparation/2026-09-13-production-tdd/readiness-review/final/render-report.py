from pathlib import Path
import html
import re

ROOT = Path(__file__).resolve().parent
CURRENT_MAIN = False
STYLE = """html{color:#202124;background:#fff;font:17px/1.6 system-ui,sans-serif}body{max-width:1100px;margin:48px auto;padding:0 24px}h1{font-size:2.25rem;line-height:1.2}h2{margin-top:2.5rem;font-size:1.45rem}h3{font-size:1.1rem}p{max-width:92ch}a{color:#1653a1;text-underline-offset:3px}nav{border-block:1px solid #ccc;padding:16px 0;margin:28px 0}nav ul{display:flex;flex-wrap:wrap;gap:10px 24px;list-style:none;padding:0;margin:0}table{border-collapse:collapse;font-size:.88rem;width:100%}th,td{padding:12px;vertical-align:top;text-align:left;border:1px solid #ccc}th{background:#eee}tr:nth-child(even){background:#fafafa}.table-wrap{overflow:auto}code{font-size:.87em;background:#f3f3f3;overflow-wrap:anywhere}pre{white-space:pre-wrap;padding:16px;background:#f5f5f5}li{margin:.35rem 0}:target{scroll-margin-top:18px}a:focus-visible{outline:3px solid #1653a1;outline-offset:3px}@media(max-width:650px){body{margin:24px auto;padding:0 16px}h1{font-size:1.8rem}table{min-width:700px}}@media print{body{max-width:none;margin:0;padding:0;font-size:10pt}nav{display:none}h2{break-after:avoid}tr{break-inside:avoid}.table-wrap{overflow:visible}table{min-width:0;font-size:8pt}a{color:inherit}}"""
PAGES = [
    ("report", "Assessment"),
    ("capability-family-preservation-ledger", "23 capability families"),
    ("native-nonpayable-usability-protocol", "12 usability tasks"),
    ("security-sources", "Security sources"),
    ("operations-sources", "Operations sources"),
    ("capability-sources", "Foundry sources"),
]


def inline(value):
    value = html.escape(value)
    protected = []

    def hold(fragment):
        protected.append(fragment)
        return f"\x00{len(protected) - 1}\x00"

    value = re.sub(r"`([^`]+)`", lambda m: hold("<code>" + m[1] + "</code>"), value)
    value = re.sub(r"\[([^\]]+)\]\((https?://[^)]+)\)", lambda m: hold('<a href="' + m[2] + '">' + m[1] + "</a>"), value)
    value = re.sub(r"https?://[^\s<>]+", lambda m: hold('<a href="' + m[0].rstrip(".,;") + '">' + m[0].rstrip(".,;") + "</a>" + m[0][len(m[0].rstrip(".,;")):]), value)
    value = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", value)
    if CURRENT_MAIN:
        def citation(match):
            label = match[1]
            linked = re.sub(r"\d+", lambda n: '<a href="#source-' + n[0] + '">' + n[0] + '</a>', label)
            return '<sup>' + linked + '</sup>'
        value = re.sub(r"\[(\d+(?:[,–-]\d+)*)\]", citation, value)
    return re.sub(r"\x00(\d+)\x00", lambda m: protected[int(m[1])], value)


def render(name):
    global CURRENT_MAIN
    CURRENT_MAIN = name == "report"
    lines = (ROOT / (name + ".txt")).read_text().splitlines()
    title = lines[0].lstrip("# ").strip()
    links = [(n + ".html", label) for n, label in PAGES] + [("postgres-update.json", "Database verification"), ("credential-oracle-update.json", "Checker verification")]
    nav = '<nav aria-label="Research package"><ul>' + "".join('<li><a href="' + f + '">' + label + "</a></li>" for f, label in links) + "</ul></nav>"
    parts, index = [], 0
    while index < len(lines):
        line = lines[index].strip()
        if not line:
            index += 1
            continue
        if line.startswith("#"):
            level = len(line) - len(line.lstrip("#"))
            parts.append(f"<h{level}>" + inline(line[level:].strip()) + f"</h{level}>")
            index += 1
            if level == 1:
                parts.append(nav)
            continue
        if line.startswith("|"):
            rows = []
            while index < len(lines) and lines[index].strip().startswith("|"):
                cells = [x.strip() for x in lines[index].strip().strip("|").split("|")]
                if not all(re.fullmatch(r":?-+:?", x) for x in cells):
                    rows.append(cells)
                index += 1
            parts.append('<div class="table-wrap" role="region" aria-label="Comparison table" tabindex="0"><table><thead><tr>' + "".join('<th scope="col">' + inline(c) + "</th>" for c in rows[0]) + "</tr></thead><tbody>" + "".join("<tr>" + "".join("<td>" + inline(c) + "</td>" for c in row) + "</tr>" for row in rows[1:]) + "</tbody></table></div>")
            continue
        if line.startswith("- "):
            items = []
            while index < len(lines) and lines[index].strip().startswith("- "):
                items.append("<li>" + inline(lines[index].strip()[2:]) + "</li>")
                index += 1
            parts.append("<ul>" + "".join(items) + "</ul>")
            continue
        paragraph = [line]
        index += 1
        while index < len(lines) and lines[index].strip() and not lines[index].lstrip().startswith(("#", "|", "- ")):
            paragraph.append(lines[index].strip())
            index += 1
        body = " ".join(paragraph)
        source = re.match(r"\[(\d+)\] ", body) if CURRENT_MAIN else None
        identity = ' id="source-' + source[1] + '"' if source else ""
        parts.append("<p" + identity + ">" + inline(body) + "</p>")
    page = '<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>' + html.escape(title) + "</title><style>" + STYLE + "</style></head><body><main>" + "".join(parts) + "</main></body></html>"
    (ROOT / (name + ".html")).write_text(page)


if __name__ == "__main__":
    for name, _ in PAGES:
        render(name)
    print("Rendered six local research pages; no scripts or external assets.")
