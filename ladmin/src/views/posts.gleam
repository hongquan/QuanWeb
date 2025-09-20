import gleam/list
import gleam/string
import lustre/attribute as a
import lustre/element/html as h
import tempo.{DateFormat}
import tempo/datetime

import core.{type Post, PageOwnedPosts, Post}
import models

const class_cell = "px-4 py-4 text-sm font-medium whitespace-nowrap"

pub fn render_post_table_view(_page: Int, model: models.Model) {
  let assert PageOwnedPosts(posts) = model.page_owned_objects
  let rows = posts |> list.map(render_post_row)
  h.div(
    [
      a.class(
        "overflow-x-auto border border-gray-200 dark:border-gray-700 md:rounded-lg",
      ),
    ],
    [
      h.table(
        [a.class("min-w-full divide-y divide-gray-200 dark:divide-gray-700")],
        [render_post_table_header(), h.tbody([], rows)],
      ),
    ],
  )
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
  h.thead([a.class("bg-gray-50 dark:bg-gray-800")], [
    h.tr([], cells),
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
    h.td([a.class(class_cell)], [h.text(post.slug)]),
    h.td([a.class(class_cell)], [h.text(categories)]),
    h.td([a.class(class_cell)], [h.text(created_at_str)]),
  ])
}
