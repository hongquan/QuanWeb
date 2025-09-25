import gleam/list
import gleam/string
import lustre/attribute as a
import lustre/element/html as h
import tempo.{DateFormat}
import tempo/datetime

import core.{type Post, PageOwnedPosts, Post}
import icons/heroicons.{globe_asia_australia}
import lucide_lustre as lucide_icon
import models.{type Model}
import views/load_indicators.{render_three_bar_pulse}
import views/skeleton
import views/ui_components.{render_paginator}

const class_cell = "px-4 py-4"

pub fn render_post_table_view(page: Int, model: Model) {
  case model.is_loading {
    True ->
      skeleton.render_main_block([
        h.div([a.class("mt-12")], [render_three_bar_pulse()]),
      ])
    False -> {
      let assert PageOwnedPosts(posts) = model.page_owned_objects
      let total_pages = model.page_owned_object_paging.total_pages
      let paginator = render_paginator(page, total_pages, [])
      let rows = posts |> list.map(render_post_row)
      let body =
        h.div(
          [
            a.class(
              "overflow-x-auto border border-gray-200 dark:border-gray-700 md:rounded-lg",
            ),
          ],
          [
            h.table(
              [a.class("w-full divide-y divide-gray-200 dark:divide-gray-700")],
              [
                render_post_table_header(),
                h.tbody(
                  [
                    a.class(
                      "bg-white divide-y divide-gray-200 dark:divide-gray-700 dark:bg-gray-900",
                    ),
                  ],
                  rows,
                ),
              ],
            ),
          ],
        )
      skeleton.render_main_block([body, paginator])
    }
  }
}

fn render_post_table_header() {
  let columns = ["Title", "Slug", "Categories", "Created"]
  let cells =
    columns
    |> list.map(fn(label) {
      h.th(
        [
          a.class(
            "py-3.5 px-4 text-sm font-normal text-left rtl:text-right text-gray-500 dark:text-gray-400",
          ),
          a.scope("col"),
        ],
        [h.text(label)],
      )
    })
  let action_column = h.th([a.class("sr-only")], [h.text("Action")])
  h.thead([a.class("bg-gray-50 dark:bg-gray-800")], [
    h.tr([], cells |> list.append([action_column])),
  ])
}

fn render_post_row(post: Post) {
  let Post(title:, created_at:, ..) = post
  let created_at_str =
    datetime.format(created_at, DateFormat(tempo.CustomDate("DD MMM YYYY")))
  let categories =
    post.categories |> list.map(fn(c) { c.title }) |> string.join(", ")
  h.tr([], [
    h.td([a.class(class_cell)], [h.text(title)]),
    h.td([a.class(class_cell), a.class("text-sm")], [h.text(post.slug)]),
    h.td([a.class(class_cell), a.class("text-sm")], [h.text(categories)]),
    h.td([a.class(class_cell), a.class("text-sm")], [h.text(created_at_str)]),
    h.td([a.class(class_cell), a.class("text-sm")], [
      h.div([a.class("flex items-center space-x-4")], [
        h.a(
          [
            a.href("#"),
            a.class("block w-5 h-auto text-green-600 hover:text-green-400"),
          ],
          [globe_asia_australia()],
        ),
        h.a([a.href("#"), a.class("hover:text-blue-600")], [
          lucide_icon.view([a.class("w-5 h-auto")]),
        ]),
        h.a([a.href("#"), a.class("hover:text-red-600")], [
          lucide_icon.eraser([a.class("w-5 h-auto")]),
        ]),
      ]),
    ]),
  ])
}
