import gleam/int
import gleam/list
import gleam/uri
import iterators.{type Iterator}
import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/element/keyed
import routes

const class_active = "text-blue-600 border-blue-300 bg-blue-50 hover:bg-blue-100 hover:text-blue-700 dark:bg-gray-700 dark:text-white dark:hover:bg-blue-500 dark:hover:text-gray-200"

const class_inactive = "hidden sm:block text-gray-500 border-gray-300 bg-white hover:bg-gray-100 hover:text-gray-700 dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400 dark:hover:bg-gray-700 dark:hover:text-white"

const class_ellipsis = "hidden sm:block text-gray-500 border-gray-300 bg-white dark:bg-gray-800 dark:border-gray-700"

pub type PageLink {
  PageLink(
    label: String,
    page: Int,
    is_ellipsis: Bool,
    is_current: Bool,
    css_class: String,
  )
}

const default_page_link = PageLink("", 1, False, False, "")

pub fn render_paginator(
  current_page: Int,
  total: Int,
  url_query: List(#(String, String)),
) -> Element(msg) {
  let links =
    gen_links(current_page, total)
    |> iterators.map(fn(lk) {
      #(int.to_string(lk.page), make_html_from_link(lk, url_query))
    })
  let prev_page = int.clamp(current_page - 1, 1, total)
  let prev_query_string =
    url_query |> routes.replace_page(prev_page) |> uri.query_to_string
  let prev_link = #(
    "prev",
    h.a(
      [
        a.class(
          "block px-3 py-2 ms-0 leading-tight text-gray-500 bg-white border border-gray-300 rounded-l-lg dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400",
        ),
        a.classes([
          #("cursor-not-allowed", current_page == 1),
          #(
            "hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-700 dark:hover:text-white",
            current_page != 1,
          ),
        ]),
        a.href("?" <> prev_query_string),
      ],
      [
        h.span([a.class("sr-only")], [h.text("Previous")]),
        h.span([], [h.text("❮")]),
      ],
    ),
  )
  let next_page = int.clamp(current_page + 1, 1, total)
  let next_query_string =
    url_query |> routes.replace_page(next_page) |> uri.query_to_string
  let next_link = #(
    "next",
    h.a(
      [
        a.class(
          "block px-3 py-2 leading-tight text-gray-500 bg-white border border-gray-300 rounded-r-lg dark:bg-gray-800 dark:border-gray-700 dark:text-gray-400",
        ),
        a.classes([
          #("cursor-not-allowed", current_page == total),
          #(
            "hover:bg-gray-100 hover:text-gray-700 dark:hover:bg-gray-700 dark:hover:text-white",
            current_page != total,
          ),
        ]),
        a.href("?" <> next_query_string),
      ],
      [
        h.span([a.class("sr-only")], [h.text("Previous")]),
        h.span([], [h.text("❯")]),
      ],
    ),
  )
  let links =
    prev_link
    |> list.wrap
    |> iterators.from_list
    |> iterators.chain(links)
    |> iterators.chain({ next_link |> list.wrap |> iterators.from_list })
    |> iterators.to_list
  keyed.element(
    "nav",
    [a.class("relative inline-flex mt-6 rounded-md shadow-sm -space-x-px")],
    links,
  )
}

fn gen_links(current_page: Int, total: Int) -> Iterator(PageLink) {
  let padding = 3
  let max_display = padding * 2 + 1
  case total <= max_display {
    True -> {
      iterators.naturals(1)
      |> iterators.take(total)
      |> iterators.map(fn(page) {
        let is_current = page == current_page
        let css_class = case is_current {
          True -> class_active
          False -> class_inactive
        }
        PageLink(
          ..default_page_link,
          page:,
          label: int.to_string(page),
          is_current:,
          css_class:,
        )
      })
    }
    False -> {
      let ellipsis_index = case
        current_page == padding,
        current_page == total - padding + 1
      {
        True, _ -> current_page + 1
        False, True -> max_display - padding - 2
        _, _ -> padding
      }
      iterators.naturals(0)
      |> iterators.take(max_display)
      |> iterators.map(fn(i) {
        let #(is_ellipsis, page) = case
          current_page <= padding || current_page > total - padding
        {
          // When current page is around beginning or around the end, we show one ellipsis
          True -> {
            let page = case i < ellipsis_index {
              True -> i + 1
              False -> total - 2 * padding + i
            }
            #(i == ellipsis_index, page)
          }
          // When current page is at the middle, we show two ellipses.
          // We don't base on ellipsisIndex anymore.
          _ -> {
            let page = case i {
              // Always show first page
              0 -> 1
              // Always show last page
              x if x >= max_display - 1 -> total
              // Show closest neighbor pages
              _ -> current_page - padding + i
            }
            let is_ellipsis = i == 1 || i == max_display - 2
            #(is_ellipsis, page)
          }
        }
        let label = case is_ellipsis {
          True -> "…"
          False -> int.to_string(page)
        }
        let is_current = page == current_page
        let css_class = case is_current {
          True -> class_active
          False -> class_inactive
        }
        PageLink(page:, is_ellipsis:, label:, is_current:, css_class:)
      })
    }
  }
}

fn make_html_from_link(
  lk: PageLink,
  url_query: List(#(String, String)),
) -> Element(a) {
  case lk.is_ellipsis {
    True -> h.span([a.class(class_ellipsis)], [h.text("…")])
    False -> {
      let aria_current = case lk.is_current {
        True -> a.aria_current("page")
        _ -> a.none()
      }
      let new_query =
        url_query |> routes.replace_page(lk.page) |> uri.query_to_string
      h.a(
        [
          a.class("px-3 py-2 leading-tight min-w-8"),
          a.class(lk.css_class),
          a.href("?" <> new_query),
          aria_current,
        ],
        [h.text(lk.label)],
      )
    }
  }
}
