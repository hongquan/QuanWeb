import gleam/list
import gleam/option.{None, Some}

import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/element/keyed
import lustre/event as ev

import core.{
  type Presentation, ContentItemDeletionClicked, IsLoading, LogOutClicked,
  PageOwnedPresentations, Presentation, PresentationId,
}
import lucide_lustre as lucide_icon
import models.{type Model, Model}
import routes.{PresentationEditPage}
import views/forms
import views/load_indicators.{render_three_bar_pulse}
import views/skeleton
import views/ui_components.{render_flash_messages, render_paginator}

const class_cell = "px-4 py-4"

// Presentation list page
pub fn render_presentation_table_page(page: Int, model: Model) {
  let Model(route:, loading_status:, ..) = model
  case loading_status {
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
      let presentations = case model.page_owned_objects {
        PageOwnedPresentations(objects) -> objects
        _ -> []
      }
      let total_pages = model.page_owned_object_paging.total_pages
      let paginator = render_paginator(page, total_pages, [])
      let deletion_handler = fn(id) {
        ContentItemDeletionClicked(PresentationId(id))
      }

      let rows =
        presentations
        |> list.map(render_presentation_row(_, deletion_handler))

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
                render_presentation_table_header(),
                table_body,
              ],
            ),
          ],
        )

      element.fragment([
        skeleton.render_header_bar(LogOutClicked),
        skeleton.render_tab_navbar(route),
        skeleton.render_main_block(
          [
            render_flash_messages(model.flash_messages),
            body,
            paginator,
          ],
          "space-y-8",
        ),
      ])
    }
  }
}

fn render_presentation_table_header() {
  let columns = ["Title", "URL", "Event"]
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

fn render_presentation_row(
  presentation: Presentation,
  deletion_click_handler: fn(String) -> msg,
) -> #(String, Element(msg)) {
  let Presentation(id:, title:, url:, event:) = presentation
  let event_text = case event {
    Some(e) -> e
    None -> "-"
  }
  let edit_url = routes.as_url_string(PresentationEditPage(id))
  let cells = [
    h.td([a.class(class_cell)], [
      h.a([a.href(edit_url), a.class("hover:underline")], [h.text(title)]),
    ]),
    h.td([a.class(class_cell), a.class("text-sm")], [
      h.a([a.href(url), a.target("_blank"), a.class("hover:underline")], [
        h.text(url),
      ]),
    ]),
    h.td([a.class(class_cell), a.class("text-sm")], [h.text(event_text)]),
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

pub fn render_presentation_edit_page(id: String, model: Model) {
  let Model(route:, loading_status:, ..) = model
  case loading_status {
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
      let form = case model.presentation_form, id {
        Some(form), "" ->
          forms.render_presentation_form(None, form, loading_status)
        Some(form), pid ->
          forms.render_presentation_form(Some(pid), form, loading_status)
        _, _ -> element.none()
      }
      element.fragment([
        skeleton.render_header_bar(LogOutClicked),
        skeleton.render_tab_navbar(route),
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
