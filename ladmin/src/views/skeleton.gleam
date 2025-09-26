import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/event as ev

pub fn render_main_block(children: List(Element(msg))) -> Element(msg) {
  h.main([a.class("mx-auto w-full max-w-320 mt-8")], children)
}

pub fn render_header_bar(logout_click_msg: msg) -> Element(msg) {
  let class_item_link =
    "text-gray-700 transition-colors duration-300 transform dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 hover:underline"
  h.header([a.class("mx-auto w-full max-w-320 pt-6")], [
    h.div([a.class("sm:flex item-center justify-between")], [
      h.a([a.class("text-3xl")], [h.text("Admin")]),
      h.nav([a.class("space-x-4")], [
        h.a(
          [
            a.href("/"),
            a.class(class_item_link),
          ],
          [h.text("View site")],
        ),
        h.button(
          [
            a.class(class_item_link),
            a.class("cursor-pointer"),
            ev.on_click(logout_click_msg),
          ],
          [h.text("Logout")],
        ),
      ]),
    ]),
  ])
}
