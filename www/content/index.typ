#import "../utils/page.typ": page
#import "../utils/ui.typ": hero
#import "../utils/site.typ": NAME, SITE

#let meta = (
  route: "/",
  label: "Overview",
  title: NAME,
  summary: SITE.tagline,
  home: true,
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

#hero()

= Docs in progress

Pages land here as they get written. Until then, the
#link("https://github.com/mlavrinenko/canibump#readme")[README] has the tour.
