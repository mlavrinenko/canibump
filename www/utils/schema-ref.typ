// Renders a JSON Schema that `canibump schema` printed, so a reference page
// states each key once: in the doc comment on its Rust type. www/generate.sh
// writes www/generated/schema-<name>.json for `manifest`, `report` and
// `state`.
//
// A page calls `reference(name, root, sections)`. `root` is the heading of the
// top-level keys, and `sections` pairs each definition with the heading it
// renders under, in reading order. A definition whose variants share keys
// (`Blocker`, `Step`) renders the shared keys once, then each variant's own.
// The build fails on a definition the page leaves out, so a new type can't
// skip the reference.

#import "../templates/base.typ": slug

#let _def-name(ref) = ref.split("/").last()

// The first paragraph of a description, with `code` spans as raw. The rest of
// a doc comment is for whoever reads the Rust.
#let _desc(text) = {
  // A rustdoc link means nothing to a reader of the schema.
  let link = text.find(regex("\[`[^`]+`\]"))
  assert(link == none, message: "a doc comment links " + repr(link) + "; describe it in words")
  let para = text.split("\n\n").first().replace("\n", " ")
  let parts = para.split("`")
  for (i, part) in parts.enumerate() {
    if calc.odd(i) { raw(part) } else { part }
  }
}

// A key's type, linking each definition to the heading it renders under.
#let _type(node, link-to, object) = {
  if "$ref" in node { return link-to(_def-name(node.at("$ref"))) }
  if "anyOf" in node {
    let parts = node.anyOf.filter(n => n.at("type", default: none) != "null")
    return parts.map(p => _type(p, link-to, object)).join[ or ]
  }
  if "oneOf" in node {
    return node.oneOf.map(n => raw(repr(n.const))).join[, ]
  }
  let kind = node.at("type", default: none)
  if type(kind) == array {
    kind = kind.filter(k => k != "null")
    kind = if kind.len() == 1 { kind.first() } else { kind.join(" or ") }
  }
  if kind == "array" { return [array of #_type(node.items, link-to, object)] }
  if kind == "object" { return object }
  if kind == "integer" { return "integer" }
  kind
}

#let _fields(def, keys, link-to, object) = {
  let required = def.at("required", default: ())
  table(
    columns: 3,
    table.header[Key][Type][What it is],
    ..keys
      .map(key => {
        let prop = def.properties.at(key)
        let kind = _type(prop, link-to, object)
        (
          raw(key),
          if key in required [#kind, required] else { kind },
          _desc(prop.at("description", default: "")),
        )
      })
      .flatten(),
  )
}

#let _values(def) = table(
  columns: 2,
  table.header[Value][What it means],
  ..def.oneOf.map(v => (raw(repr(v.const)), _desc(v.description))).flatten(),
)

// The keys every variant has, in the first variant's order.
#let _shared(variants) = {
  let keys = variants.first().properties.keys()
  keys.filter(key => variants.all(v => key in v.properties))
}

// The variants of an `anyOf` definition: each a table of keys, or a bare
// value like a string.
#let _variants(schema, def) = def.anyOf.map(v => (
  desc: v.at("description", default: ""),
  def: if "$ref" in v { schema.at("$defs").at(_def-name(v.at("$ref"))) } else { v },
))

// A variant's own keys: those its siblings don't all share.
#let _own(entry, shared) = entry.def.at("properties", default: (:)).keys().filter(k => k not in shared)

#let _section(schema, def, link-to, object, earlier: none) = {
  if "description" in def { par(_desc(def.description)) }
  if "oneOf" in def {
    _values(def)
    return
  }
  if "anyOf" in def {
    let entries = _variants(schema, def)
    let tables = entries.filter(e => "properties" in e.def).map(e => e.def)
    let shared = if tables.len() > 1 { _shared(tables) } else { () }
    if shared != () { _fields(tables.first(), shared, link-to, object) }
    // Variants an earlier section already listed, with the same own keys,
    // link back to it.
    if earlier != none {
      par[Plus exactly one condition, with the same keys as in #link-to(earlier).]
      return
    }
    for entry in entries {
      if "properties" in entry.def {
        par(strong(_desc(entry.desc)))
        _fields(entry.def, _own(entry, shared), link-to, object)
      } else {
        par[#_type(entry.def, link-to, object): #_desc(entry.desc)]
      }
    }
    return
  }
  _fields(def, def.properties.keys(), link-to, object)
}

#let reference(name, root, sections, object: "table") = {
  let schema = json("../generated/schema-" + name + ".json")
  let defs = schema.at("$defs", default: (:))
  let headings = sections.to-dict()
  let link-to(def) = link("#" + slug(headings.at(def)), raw(headings.at(def)))

  // Every definition renders, as a section or as a variant of one.
  let variants = sections
    .map(((def, _)) => defs.at(def).at("anyOf", default: ()))
    .flatten()
    .filter(v => "$ref" in v)
    .map(v => _def-name(v.at("$ref")))
  let missing = defs.keys().filter(d => d not in headings and d not in variants)
  assert(missing == (), message: name + " schema: no section renders " + missing.join(", "))

  heading(raw(root))
  if "description" in schema { par(_desc(schema.description)) }
  _fields(schema, schema.properties.keys(), link-to, object)
  // A section whose variants have the same own keys, in order, as an earlier
  // section's links back to it rather than repeat its tables.
  let signature(def) = {
    let entries = _variants(schema, defs.at(def))
    let tables = entries.filter(e => "properties" in e.def).map(e => e.def)
    let shared = if tables.len() > 1 { _shared(tables) } else { () }
    entries.map(e => _own(e, shared))
  }
  let seen = (:)
  for (def, title) in sections {
    heading(raw(title))
    let earlier = none
    if "anyOf" in defs.at(def) {
      let sig = repr(signature(def))
      earlier = seen.at(sig, default: none)
      if earlier == none { seen.insert(sig, def) }
    }
    _section(schema, defs.at(def), link-to, object, earlier: earlier)
  }
}
