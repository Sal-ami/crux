#!/usr/bin/env python3
"""generates the crux docs pages into docs/. edit PAGES, run: python gen.py"""
import pathlib

OUT = pathlib.Path(__file__).parent / "docs"

from pages_content import PAGES

GROUP_ORDER = ["getting started", "features", "how it works", "guarding behaviors", "reference"]

TEMPLATE_HEAD = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} - crux</title>
<meta name="description" content="crux documentation. {title}.">
<link rel="canonical" href="https://crux.rweb.site/docs/{page}">
<meta property="og:title" content="{title} - crux">
<meta property="og:description" content="crux documentation. {title}.">
<meta property="og:image" content="https://crux.rweb.site/og.png">
<meta property="og:type" content="article">
<meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)">
<meta name="theme-color" content="#000000" media="(prefers-color-scheme: dark)">
<link rel="icon" href="../favicon.svg">
<link rel="alternate" type="application/rss+xml" title="crux changelog" href="/feed.xml">
<link rel="stylesheet" href="style.css">
</head>
<body>
<a class="skip-link" href="#content">skip to content</a>

<header>
  <a class="logo" href="../">crux</a>
  <nav>
    <a href="../">home</a>
    <a href="https://github.com/Emran-goat/crux" target="_blank" rel="noreferrer">source</a>
    <a class="nav-cta" href="../#support">support us</a>
  </nav>
</header>

<div class="layout">
  <aside class="sidenav">{nav}</aside>
  <main id="content">{content}
    <a class="edit-link" href="https://github.com/Emran-goat/crux/blob/main/landing/pages_content.py" target="_blank" rel="noreferrer">edit this page</a>
  </main>
  <aside class="pagetoc">
    <div class="toc-label">on this page</div>
    <div id="toc-links"></div>
  </aside>
</div>
"""

TEMPLATE_TAIL = """<script>
  (function () {
    var heads = document.querySelectorAll("main h2[id]");
    var box = document.getElementById("toc-links");
    var wrap = document.querySelector(".pagetoc");
    if (!heads.length || !box) { if (wrap) wrap.style.display = "none"; return; }
    var links = [];
    heads.forEach(function (h) {
      var a = document.createElement("a");
      a.href = "#" + h.id;
      a.textContent = h.textContent;
      box.appendChild(a);
      links.push([h, a]);
    });
    var current = null;
    var io = new IntersectionObserver(function (entries) {
      entries.forEach(function (e) {
        if (e.isIntersecting && current !== null || e.isIntersecting) {
          links.forEach(function (pair) {
            if (pair[0] === e.target && e.isIntersecting) {
              if (current) current.classList.remove("active");
              pair[1].classList.add("active");
              current = pair[1];
            }
          });
        }
      });
    }, { rootMargin: "0px 0px -70% 0px" });
    heads.forEach(function (h) { io.observe(h); });
  })();
</script>
<script src="copy.js"></script>

</body>
</html>
"""
def build_nav(active_slug):
    parts = []
    for group in GROUP_ORDER:
        pages = [p for p in PAGES if p[2] == group]
        if not pages:
            continue
        parts.append('<div class="group"><div class="group-label">%s</div>' % group)
        for slug, title, _, _ in pages:
            href = ("./" if slug == "" else "./" + slug + ".html")
            cls = ' class="active" aria-current="page"' if slug == active_slug else ""
            parts.append('<a%s href="%s">%s</a>' % (cls, href, title))
        parts.append("</div>")
    return "".join(parts)


def main():
    for slug, title, group, content in PAGES:
        page = "index.html" if slug == "" else slug + ".html"
        body = content.replace("<img ", '<img loading="lazy" ')
        html = TEMPLATE_HEAD.format(title=title, page=page, nav=build_nav(slug), content=body)
        html += TEMPLATE_TAIL
        out = OUT / page
        out.write_text(html, encoding="utf-8")
        print("wrote", out.name)

    base = "https://crux.rweb.site/"
    rows = ['<?xml version="1.0" encoding="UTF-8"?>',
            '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">',
            f"<url><loc>{base}</loc></url>",
            f"<url><loc>{base}install.sh</loc></url>"]
    for slug, _, _, _ in PAGES:
        loc = base + "docs/" + ("index.html" if slug == "" else slug + ".html")
        rows.append(f"<url><loc>{loc}</loc></url>")
    rows.append("</urlset>")
    (OUT / "sitemap.xml").write_text("\n".join(rows), encoding="utf-8")
    (pathlib.Path(__file__).parent / "robots.txt").write_text(
        "User-agent: *\nAllow: /\nSitemap: https://crux.rweb.site/docs/sitemap.xml\n",
        encoding="utf-8")
    (OUT / "404.html").write_text(
        TEMPLATE_HEAD.format(title="not found", page="404.html", nav=build_nav(None), content=
        '<p>That page does not exist. The sidebar lists everything that does.</p>')
        + TEMPLATE_TAIL, encoding="utf-8")
    (OUT / ".nojekyll").write_text("", encoding="utf-8")
    print("wrote sitemap.xml, robots.txt, 404.html, .nojekyll")


if __name__ == "__main__":
    main()
