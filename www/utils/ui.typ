// Rendering helpers the pages share.

#import "site.typ": MARKDOWN, NAME, REPO, SITE, md-copy-button, u

// `html.elem(tag, attrs: attrs, body)`, or the bare body in the Markdown copy
// (mode.typ), where typlite drops a div or a span together with its content.
#let el(tag, attrs: (:), body) = if MARKDOWN { body } else {
  html.elem(tag, attrs: attrs, body)
}

// A callout box, kind ∈ note | tip | warn. The Markdown copy shows a
// paragraph that opens with the kind, since typlite pulls the paragraph
// before a quote into it.
#let _callout-label = (note: "Note", tip: "Tip", warn: "Warning")
#let callout(kind, body) = if MARKDOWN {
  par[#strong(_callout-label.at(kind) + ".") #body]
} else {
  html.elem("div", attrs: (class: "callout callout-" + kind), body)
}

// The site root's opening: the name, the tagline and the ways in.
#let hero() = if MARKDOWN [
  #par(SITE.tagline)

  #link(u("/quickstart/"))[Get started] · #link(REPO)[GitHub]
] else {
  html.elem("section", attrs: (class: "hero"), {
    html.elem("div", attrs: (class: "hero-lede"), {
      html.elem(
        "div",
        attrs: (class: "page-title"),
        html.elem("h1", NAME) + md-copy-button("/"),
      )
      html.elem("p", attrs: (class: "hero-sub"), SITE.tagline)
      html.elem("div", attrs: (class: "hero-cta"), {
        html.elem("a", attrs: (class: "btn", href: u("/quickstart/")), [Get started])
        html.elem("a", attrs: (class: "btn ghost", href: REPO), [GitHub])
      })
    })
  })
}

// ── Captures ─────────────────────────────────────────────────────────────────
// www/capture.sh writes www/generated/<slug>.json: the fixture the commands
// ran in, and per command the line bash ran, what the terminal showed and the
// exit code. These helpers print each command from the record, so a page
// can't show a command other than the one that ran.

#let _record(slug) = json("../generated/" + slug + ".json")

// The colours a capture's SGR escapes ask for, One Dark-ish, readable on the
// terminal panel. Dim is canibump's `waiting`.
#let _ansi-fg = (
  "30": "#5c6370",
  "31": "#e06c75",
  "32": "#98c379",
  "33": "#e5c07b",
  "34": "#61afef",
  "35": "#c678dd",
  "36": "#56b6c2",
  "37": "#dcdfe4",
  "90": "#7f848e",
  "91": "#e06c75",
  "92": "#98c379",
  "93": "#e5c07b",
  "94": "#61afef",
  "95": "#c678dd",
  "96": "#56b6c2",
  "97": "#ffffff",
)
#let _dim = "#8b97a3"

// One SGR parameter list ("1;31", "0", ...) applied to the state, a
// (colour, bold, dim) triple.
#let _sgr-apply(state, codes) = {
  let (col, bold, dim) = state
  for p in codes.split(";") {
    if p == "0" or p == "" {
      (col, bold, dim) = (none, false, false)
    } else if p == "1" {
      bold = true
    } else if p == "2" {
      dim = true
    } else if p == "22" {
      (bold, dim) = (false, false)
    } else if p == "39" {
      col = none
    } else if p in _ansi-fg {
      col = _ansi-fg.at(p)
    }
  }
  (col, bold, dim)
}
#let _sgr-re = regex("\u{1b}\[([0-9;]*)m")

// ANSI text -> content: plain runs as text, coloured runs as styled spans,
// the way Typst's HTML export renders syntax highlighting.
#let _ansi-spans(body) = {
  let out = ()
  let pos = 0
  let state = (none, false, false)
  let run(state, text) = {
    let (col, bold, dim) = state
    let style = ()
    if col != none { style.push("color: " + col) } else if dim {
      style.push("color: " + _dim)
    }
    if bold { style.push("font-weight: bold") }
    if style == () { text } else {
      html.elem("span", attrs: (style: style.join("; ")), text)
    }
  }
  for m in body.matches(_sgr-re) {
    if m.start > pos { out.push(run(state, body.slice(pos, m.start))) }
    state = _sgr-apply(state, m.captures.at(0))
    pos = m.end
  }
  if pos < body.len() { out.push(run(state, body.slice(pos))) }
  out.join()
}
#let _ansi-strip(body) = body.replace(_sgr-re, "")

#let _transcript(record) = {
  let lines = record.steps.map(step => {
    let out = step.out.trim("\n", repeat: true)
    if out == "" { "$ " + step.cmd } else { "$ " + step.cmd + "\n" + out }
  })
  lines.join("\n")
}

// Each command of a capture after a `$` prompt, then what it printed, in the
// colours the terminal showed. The Markdown copy has no colour.
#let _panel(text, class: "terminal") = {
  if MARKDOWN {
    raw(_ansi-strip(text), block: true)
  } else {
    html.elem("div", attrs: (class: class), html.elem(
      "pre",
      html.elem("code", _ansi-spans(text)),
    ))
  }
}
#let terminal(slug) = _panel(_transcript(_record(slug)))

// A section per command of a `--help` capture, headed by the command.
#let help-sections(slug) = for step in _record(slug).steps {
  heading(raw(step.cmd.replace(" --help", "")))
  _panel(_ansi-strip(step.out).trim("\n", repeat: true), class: "terminal help")
}

// The exit code of a capture's last command.
#let exit-code(slug) = raw(str(_record(slug).steps.last().code))

// One line of what a capture printed, as inline code: the first line, colour
// stripped, that matches the regex `pattern`. The build fails when none does,
// so a quoted line can't outlive its text.
#let capture-line(slug, pattern) = {
  let lines = _record(slug)
    .steps
    .map(step => _ansi-strip(step.out).split("\n"))
    .flatten()
  let hit = lines.find(line => line.contains(regex(pattern)))
  assert(hit != none, message: "capture " + slug + " has no line matching " + pattern)
  raw(hit.trim())
}

// The same line as a quote, for one that reads as prose, like a `--help`
// paragraph. Its `code` spans stay code.
#let capture-quote(slug, pattern) = {
  let parts = capture-line(slug, pattern).text.split("`")
  let body = parts.enumerate().map(((i, part)) => if calc.odd(i) { raw(part) } else { part }).join()
  // typlite pulls the paragraph before a quote into it, so the Markdown copy
  // shows a plain paragraph.
  if MARKDOWN { par(body) } else { quote(block: true, body) }
}

// A file of the fixture a capture ran in, with its path as the caption.
#let _langs = (toml: "toml", nix: "nix", json: "json", yml: "yaml", yaml: "yaml")
#let _file-body(slug, path) = {
  let record = _record(slug)
  assert(record.project != "", message: "capture " + slug + " ran in no fixture")
  let text = read("../examples/" + record.project + "/" + path)
  let ext = path.split(".").last()
  raw(text.trim("\n", at: end), lang: _langs.at(ext, default: none), block: true)
}
#let file(slug, path) = if MARKDOWN [
  #raw(path)

  #_file-body(slug, path)
] else {
  html.elem("figure", attrs: (class: "file"), {
    html.elem("figcaption", raw(path))
    _file-body(slug, path)
  })
}

// A fixture file above the run it produced, for the site root.
#let showcase(slug, path) = {
  let face(kind, caption, body) = html.elem(
    "figure",
    attrs: (class: "face face-" + kind),
    {
      html.elem("figcaption", caption)
      html.elem("div", attrs: (class: "face-body"), body)
    },
  )
  if MARKDOWN [
    #file(slug, path)

    #terminal(slug)
  ] else {
    html.elem("div", attrs: (class: "showcase"), {
      face("file", raw(path), _file-body(slug, path))
      face("terminal", "terminal", terminal(slug))
    })
  }
}

// A file under src/ as inline code. The build fails on one that doesn't
// exist, so a page can't name a module after a rename.
#let src(path) = {
  let files = json("../generated/src-files.json")
  assert(path in files, message: "src/" + path + " does not exist")
  raw(path)
}
