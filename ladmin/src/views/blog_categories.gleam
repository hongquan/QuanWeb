import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}

import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/element/keyed
import lustre/event as ev

import core.{
  type Category, Category, IsLoading, LogOutClicked, PageOwnedCategories,
  ContentItemDeletionClicked, CategoryId,
}
import lucide_lustre as lucide_icon
import models.{type Model, Model}
import routes.{type CategorySort, CategoryEditPage, SortByFeatured}
import views/forms
import views/load_indicators.{render_three_bar_pulse}
import views/skeleton
import views/ui_components.{render_flash_messages, render_paginator}

const class_cell = "px-4 py-4"

pub fn render_category_table_page(page: Int, sort: Option(CategorySort), model: Model) {
  let Model(route:, ..) = model
  case model.loading_status {
    IsLoading ->
      element.fragment([
        skeleton.render_header_bar(LogOutClicked),
        skeleton.render_tab_navbar(route),
        skeleton.render_main_block(
          [
            h.div([a.class("mt-12 space-y-12")], [
              render_flash_messages(model.flash_messages),
              render_three_bar_pulse(),
            ]),
          ],
          "",
        ),
      ])

    _ -> {
      let categories = case model.page_owned_objects {
        PageOwnedCategories(objects) -> objects
        _ -> []
      }
      // Categories are already sorted by the server based on sort mode
      let total_pages = model.page_owned_object_paging.total_pages
      let query_list = case sort {
        Some(SortByFeatured) -> [#("sort", "featured")]
        _ -> []
      }
      let paginator = render_paginator(page, total_pages, query_list)
      let deletion_handler = fn(id) {
        ContentItemDeletionClicked(CategoryId(id))
      }

      let rows =
        categories
        |> list.map(render_category_row(_, deletion_handler))

      let table_body =
        keyed.tbody(
          [
            a.class(
              "bg-white divide-y divide-y-reverse divide-gray-200 dark:divide-gray-700 dark:bg-gray-900",
            ),
          ],
          rows,
        )

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
                render_category_table_header(),
                table_body,
              ],
            ),
          ],
        )

      let url_new_category = routes.as_url_string(CategoryEditPage(""))
      let link_create_category =
        h.a(
          [
            a.href(url_new_category),
            a.class(
              "px-6 py-2 font-medium tracking-wide text-white capitalize transition-colors duration-300 transform bg-blue-600 rounded-lg hover:bg-blue-500 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-80",
            ),
          ],
          [h.text("New category")],
        )

      // Sort mode toggle buttons - navigate to URL with sort parameter
      let sort_by_title_active = sort == None
      let sort_by_featured_active = sort == Some(SortByFeatured)
      let title_sort_url = routes.as_url_string(routes.CategoryListPage(Some(page), None))
      let featured_sort_url = routes.as_url_string(routes.CategoryListPage(Some(page), Some(SortByFeatured)))
      let sort_toggle =
        h.div([a.class("flex items-center space-x-2 text-sm")], [
          h.span([a.class("text-gray-500 dark:text-gray-400")], [h.text("Sort by:")]),
          h.a(
            [
              a.href(title_sort_url),
              a.class(case sort_by_title_active {
                True -> "px-3 py-1 rounded-md bg-blue-600 text-white"
                False -> "px-3 py-1 rounded-md bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
              }),
            ],
            [h.text("Title")],
          ),
          h.a(
            [
              a.href(featured_sort_url),
              a.class(case sort_by_featured_active {
                True -> "px-3 py-1 rounded-md bg-blue-600 text-white"
                False -> "px-3 py-1 rounded-md bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600"
              }),
            ],
            [h.text("Featured")],
          ),
        ])

      element.fragment([
        skeleton.render_header_bar(LogOutClicked),
        skeleton.render_tab_navbar(route),
        skeleton.render_main_block(
          [
            render_flash_messages(model.flash_messages),
            h.div([a.class("flex justify-between items-center")], [
              sort_toggle,
              link_create_category,
            ]),
            body,
            paginator,
          ],
          "space-y-8",
        ),
      ])
    }
  }
}

fn render_category_table_header() {
  let columns = ["Title", "Slug", "Title (Vi)", "Order"]
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

fn render_category_row(
  category: Category,
  deletion_click_handler: fn(String) -> msg,
) -> #(String, Element(msg)) {
  let Category(id:, title:, slug:, title_vi:, featured_order:, ..) = category
  let url = routes.as_url_string(CategoryEditPage(id))
  let order_text = case featured_order {
    Some(order) -> int.to_string(order)
    None -> "-"
  }
  let cells = [
    h.td([a.class(class_cell)], [
      h.a([a.href(url), a.class("hover:underline")], [
        h.text(title),
      ]),
    ]),
    h.td([a.class(class_cell), a.class("text-sm")], [h.text(slug)]),
    h.td([a.class(class_cell), a.class("text-sm")], [
      h.text(title_vi |> option.unwrap("")),
    ]),
    h.td([a.class(class_cell), a.class("text-sm text-center w-16")], [
      h.text(order_text),
    ]),
    h.td([a.class(class_cell), a.class("text-sm")], [
      h.button(
        [
          a.class("hover:text-red-600 cursor-pointer"),
          ev.on_click(deletion_click_handler(id)),
        ],
        [
          lucide_icon.eraser([a.class("w-5 h-auto")]),
        ],
      ),
    ]),
  ]

  #(id, h.tr([], cells))
}

pub fn render_category_edit_page(id: String, model: Model) {
  let Model(category_form:, loading_status:, ..) = model
  case loading_status {
    IsLoading -> {
      element.fragment([
        skeleton.render_tab_navbar(model.route),
        skeleton.render_main_block(
          [
            h.div([a.class("mt-12 space-y-12")], [
              render_flash_messages(model.flash_messages),
              render_three_bar_pulse(),
            ]),
          ],
          "",
        ),
      ])
    }

    _ -> {
      let form = case category_form, id {
        Some(form), "" -> {
          forms.render_category_form(None, form, loading_status)
        }
        Some(form), cid -> {
          forms.render_category_form(Some(cid), form, loading_status)
        }
        _, _ -> element.none()
      }
      element.fragment([
        skeleton.render_header_bar(LogOutClicked),
        skeleton.render_tab_navbar(model.route),
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
