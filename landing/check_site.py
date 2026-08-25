import sys, re, os
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
root = os.path.dirname(os.path.abspath(__file__))

# landing page
os.chdir(root)
txt = open("index.html", encoding="utf-8").read()
hrefs = re.findall(r'(?:href|src)="([^"#]+)"', txt)
broken = [h for h in hrefs if not h.startswith(("http", "data:", "mailto:")) and not os.path.exists(h.split("#")[0])]
print("landing broken local refs:", broken if broken else 0)
for bad in ("\u00c3", "\u00c2", "\u2013\u00e2"):
    if bad in txt:
        print("landing mojibake:", repr(bad))
print("landing og.png:", os.path.exists("og.png"), "| feed.xml:", os.path.exists("feed.xml"),
      "| install.sh:", os.path.exists("install.sh"), "| robots.txt:", os.path.exists("robots.txt"))

# docsify mirror
td = os.path.join(os.path.dirname(root), "test-docs")
os.chdir(td)
broken = []
for p in [f for f in os.listdir(".") if f.endswith(".md")]:
    t = open(p, encoding="utf-8").read()
    for m in re.findall(r"\]\(([^)#]+)\)", t):
        if not m.startswith("http") and not os.path.exists(m):
            broken.append(f"{p} -> {m}")
    for m in re.findall(r"!\[[^\]]*\]\(([^)]+)\)", t):
        if not os.path.exists(m):
            broken.append(f"{p} IMG -> {m}")
print("docsify broken links/imgs:", broken if broken else 0)
