import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h

pub fn render_main_block(children: List(Element(msg))) -> Element(msg) {
  h.main([a.class("mx-auto w-full max-w-320")], children)
}
