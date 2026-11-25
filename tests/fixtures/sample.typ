#import "@preview/cetz:0.5.3"
#set page(width: auto, height: auto, margin: 0.5cm)

#cetz.canvas({
  import cetz.draw: *
  rect((0, 0), (3, -1.5), name: "n1", fill: rgb("#e3f2fd"), radius: 0.2)
  content("n1", [Start])
  circle((6.5, -0.75), radius: (1.5, 0.75), name: "n2", fill: rgb("#34c75980"))
  content("n2", [$x^2 + y^2$])
  polygon((6.5, -3.75), 4, radius: (1.5, 0.75), name: "n3", stroke: (paint: rgb("#1e88e5"), thickness: 2pt))
  content("n3", [ok?])
  content((0, -3), (3, -4.5), text(size: 14pt, weight: "bold", style: "italic", fill: rgb("#ff3b30"))[note], name: "t1")
  content((10, 0), (12, -1), image("logo.png", width: 2cm, height: 1cm, fit: "stretch"), name: "i1")
  line((name: "n1", anchor: 0deg), (name: "n2", anchor: -180deg), name: "e1", mark: (end: "stealth", fill: rgb("#000000")))
  content("e1.mid", [$alpha$], frame: "rect", fill: white, stroke: none, padding: 0.1)
  line((name: "n2", anchor: 0deg), (8.5, -0.75), (8.5, -3.75), (name: "n3", anchor: 0deg), stroke: (paint: rgb("#000000"), thickness: 1pt, dash: "dotted"), mark: (start: "o", end: ">", fill: rgb("#000000")))
  bezier((0, -6), (name: "n3", anchor: -127.569deg), (2, -5), (4, -7), stroke: (paint: rgb("#000000"), thickness: 0.5pt), mark: (end: "straight", fill: rgb("#000000")))
})
