import gleam/list
import gleam/option.{None}
import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/event as ev

import routes.{type Route, CategoryListPage, PostListPage, are_routes_matched}

pub fn render_main_block(
  children: List(Element(msg)),
  extra_class: String,
) -> Element(msg) {
  h.main(
    [
      a.class("mx-auto px-4 sm:px-0 w-full max-w-320 mt-8"),
      a.class(extra_class),
    ],
    children,
  )
}

pub fn render_header_bar(logout_click_msg: msg) -> Element(msg) {
  let class_item_link =
    "text-gray-700 transition-colors duration-300 transform dark:text-gray-200 hover:text-blue-600 dark:hover:text-blue-400 hover:underline"
  h.header(
    [
      a.class(
        "mx-auto px-4 sm:px-0 sm:w-full max-w-320 pt-6 border-b border-gray-200 dark:border-gray-600",
      ),
    ],
    [
      h.div([a.class("flex flex-row item-center justify-between")], [
        h.a([a.class("text-3xl block"), a.href("/ladmin/")], [h.text("Admin")]),
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
    ],
  )
}

pub fn render_tab_navbar(current_route: Route) {
  let class_link_common =
    "inline-flex items-center h-10 px-4 -mb-px text-sm text-center text-blue-600 bg-transparent border-b-2 sm:text-base  whitespace-nowrap focus:outline-none"
  let class_link_active =
    " border-blue-500  dark:border-blue-400 dark:text-blue-300"
  let class_link_inactive =
    " border-transparent  dark:text-white cursor-base hover:border-gray-400"
  let entries = [
    #(PostListPage(None, None, None), "Posts"),
    #(CategoryListPage(None, None), "Categories"),
  ]
  let entries =
    entries
    |> list.map(fn(kv) {
      let #(link_route, label) = kv
      h.li([], [
        h.a(
          [
            a.href(routes.as_url_string(link_route)),
            a.class(class_link_common),
            case are_routes_matched(current_route, link_route) {
              True -> a.class(class_link_active)
              False -> a.class(class_link_inactive)
            },
          ],
          [h.text(label)],
        ),
      ])
    })
  h.nav([a.class("mx-auto px-4 sm:px-0 sm:w-full max-w-320 pt-6")], [
    h.menu(
      [
        a.class(
          "flex overflow-x-auto overflow-y-hidden border-b border-gray-200 whitespace-nowrap dark:border-gray-700",
        ),
      ],
      entries,
    ),
  ])
}
