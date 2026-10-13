// Rendering helpers the pages share.

#import "site.typ": MARKDOWN, NAME, SITE, md-copy-button, u

// `html.elem(tag, attrs: attrs, body)`, or the bare body in the Markdown copy
// (mode.typ), where typlite drops a div or a span together with its content.
#let el(tag, attrs: (:), body) = if MARKDOWN { body } else {
  html.elem(tag, attrs: attrs, body)
}

// A callout box, kind ∈ note | tip | warn. The Markdown copy shows a
// blockquote that opens with the kind.
#let _callout-label = (note: "Note", tip: "Tip", warn: "Warning")
#let callout(kind, body) = if MARKDOWN {
  quote(block: true)[#strong(_callout-label.at(kind) + ":") #body]
} else {
  html.elem("div", attrs: (class: "callout callout-" + kind), body)
}

// The site root's opening: the name, the tagline and the way in.
#let hero() = if MARKDOWN { par(SITE.tagline) } else {
  html.elem("section", attrs: (class: "hero"), {
    html.elem("div", attrs: (class: "hero-lede"), {
      html.elem(
        "div",
        attrs: (class: "page-title"),
        html.elem("h1", NAME) + md-copy-button("/"),
      )
      html.elem("p", attrs: (class: "hero-sub"), SITE.tagline)
    })
  })
}
