import gleam/dynamic/decode
import gleam/javascript/array
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam/string
import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/event as ev
import plinth/browser/element as br_element
import tempo.{DateFormat}
import tempo/datetime

import core.{type Category, type Post, PageOwnedPosts, Post, PostFilterSubmitted}
import ffi
import icons/heroicons.{globe_asia_australia}
import lucide_lustre as lucide_icon
import models.{type Model, Model}
import routes.{PostEditPage}
import views/forms.{render_post_form}
import views/load_indicators.{render_three_bar_pulse}
import views/skeleton
import views/ui_components.{
  render_flash_messages, render_paginator, render_search_box,
}

const class_cell = "px-4 py-4"

pub fn render_post_table_page(
  page: Int,
  q: Option(String),
  cat_id: Option(String),
  model: Model,
) {
  case model.is_loading {
    True ->
      skeleton.render_main_block(
        [
          h.div([a.class("mt-12 space-y-12")], [
            render_flash_messages(model.flash_messages),
            render_three_bar_pulse(),
          ]),
        ],
        "",
      )
    False -> {
      let assert PageOwnedPosts(posts) = model.page_owned_objects
      let total_pages = model.page_owned_object_paging.total_pages
      let query_list = []
      let query_list = case q {
        Some(q) -> [#("q", q), ..query_list]
        _ -> query_list
      }
      let query_list = case cat_id {
        Some(q) -> [#("cat_id", q), ..query_list]
        _ -> query_list
      }
      let paginator = render_paginator(page, total_pages, query_list)
      let rows = posts |> list.map(render_post_row(_, model.mounted_path))
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
                      "bg-white divide-y divide-y-reverse divide-gray-200 dark:divide-gray-700 dark:bg-gray-900",
                    ),
                  ],
                  rows,
                ),
              ],
            ),
          ],
        )

      let initial_q = q |> option.unwrap("")
      let initial_cat_id = cat_id |> option.unwrap("")
      let url_new_post =
        routes.as_url_string(PostEditPage(""), model.mounted_path)
      element.fragment([
        skeleton.render_header_bar(core.LogOutClicked),
        skeleton.render_main_block(
          [
            render_flash_messages(model.flash_messages),
            h.a(
              [
                a.href(url_new_post),
                a.class(
                  "block sm:hidden px-6 py-2 font-medium tracking-wide text-white capitalize transition-colors duration-300 transform bg-blue-600 rounded-lg hover:bg-blue-500 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-80",
                ),
              ],
              [h.text("New post")],
            ),
            render_filter_form(
              initial_q,
              initial_cat_id,
              model.categories,
              url_new_post,
            ),
            body,
            paginator,
          ],
          "space-y-8",
        ),
      ])
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

fn render_post_row(post: Post, mounted_path: String) {
  let Post(id:, title:, slug:, created_at:, ..) = post
  let created_at_str =
    datetime.format(created_at, DateFormat(tempo.CustomDate("DD MMM YYYY")))
  let categories =
    post.categories |> list.map(fn(c) { c.title }) |> string.join(", ")
  let url = routes.as_url_string(PostEditPage(id), mounted_path)
  h.tr([], [
    h.td([a.class(class_cell)], [
      h.a([a.href(url), a.class("hover:underline")], [h.text(title)]),
    ]),
    h.td([a.class(class_cell), a.class("text-sm")], [h.text(slug)]),
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

fn render_filter_form(
  initial_q: String,
  initial_cat_id: String,
  categories: List(Category),
  url_new_post: String,
) -> Element(core.Msg(a)) {
  let choices =
    categories
    |> list.map(fn(c) {
      h.option([a.value(c.id), a.selected(c.id == initial_cat_id)], c.title)
    })
  let choices = [h.option([a.value("")], "Category..."), ..choices]
  let select_handler = {
    use el <- decode.field("target", decode.dynamic)
    let form_data =
      br_element.cast(el)
      |> result.map(ffi.get_form_data)
      |> result.map(array.to_list)
    form_data
    |> result.map(fn(data) { decode.success(PostFilterSubmitted(data)) })
    |> result.lazy_unwrap(fn() {
      decode.failure(PostFilterSubmitted([]), "FormData")
    })
  }
  let category_select =
    h.select(
      [
        a.name("cat_id"),
        a.class("border dark:border-gray-600 rounded-md py-2 ps-2 pe-4"),
        ev.on("change", select_handler),
      ],
      choices,
    )
  h.form(
    [
      a.class(
        "flex flex-col sm:flex-row space-y-4 sm:space-x-6 sm:space-y-0 text-sm",
      ),
      ev.on_submit(PostFilterSubmitted),
    ],
    [
      render_search_box(initial_q),
      category_select,
      h.div([a.class("sm:grow mb-0")], []),
      h.a(
        [
          a.href(url_new_post),
          a.class(
            "hidden sm:block px-6 py-2 font-medium tracking-wide text-white capitalize transition-colors duration-300 transform bg-blue-600 rounded-lg hover:bg-blue-500 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-80",
          ),
        ],
        [h.text("New post")],
      ),
    ],
  )
}

pub fn render_post_edit_page(_id: String, model: Model) {
  let Model(post_editing:, is_loading:, ..) = model
  case is_loading {
    True ->
      skeleton.render_main_block(
        [
          h.div([a.class("mt-12 space-y-12")], [
            render_flash_messages(model.flash_messages),
            render_three_bar_pulse(),
          ]),
        ],
        "",
      )
    False -> {
      let form = case post_editing {
        core.PostEditing(post:, form:) -> render_post_form(Some(post), form)
        core.PostCreating(form) -> render_post_form(None, form)
        _ -> element.none()
      }
      element.fragment([
        skeleton.render_header_bar(core.LogOutClicked),
        skeleton.render_main_block(
          [
            h.div([a.class("space-y-8")], [
              render_flash_messages(model.flash_messages),
              form,
            ]),
          ],
          "",
        ),
      ])
    }
  }
}
