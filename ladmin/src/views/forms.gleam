import formal/form.{type Form} as formlib
import gleam/dynamic/decode
import gleam/list
import gleam/option.{type Option, Some}
import gleam/result
import lucide_lustre as lc_icons
import lustre/attribute as a
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/event as ev
import plinth/browser/document
import plinth/browser/element as br_element

import core.{
  type Category, type CategoryEditablePart, type LoadingStatus, type MiniUser,
  type Msg, type PostEditablePart, CategoryFormSubmitted, FormCancelClicked,
  IsSubmitting, SlugGeneratorClicked, SubmitStayButtonClicked,
  UserClickMarkdownPreview, UserMovedCategoryBetweenPane,
}
import ffi
import updates
import views/widgets

const class_row = "sm:grid sm:grid-cols-4 sm:items-start sm:gap-2 sm:py-2"

const class_label = "block font-medium leading-6 dark:text-white sm:pt-2"

const class_input_col = "mt-2 sm:col-span-3 sm:mt-0"

const class_text_input = "py-2 w-full text-gray-700 bg-white border rounded-md dark:bg-gray-900 dark:text-gray-300 border-gray-300 dark:border-gray-600 focus:border-blue-400 dark:focus:border-blue-300 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-40"

const class_pane_in_field = "border border-gray-300 dark:border-gray-600 rounded-md"

const class_pane_header = "block px-2 py-1 rounded-t-md bg-gray-500 dark:bg-gray-900 text-white"

pub fn render_post_form(
  _post_id: Option(String),
  form: Form(PostEditablePart),
  categories: List(Category),
  users: List(MiniUser),
  loading_status: LoadingStatus,
) {
  let children = [
    h.div([a.class(class_row)], [
      h.label([a.class(class_label)], [h.text("Title")]),
      h.div([a.class(class_input_col)], [
        h.input([
          a.name("title"),
          a.type_("text"),
          a.required(True),
          a.value(formlib.field_value(form, "title")),
          a.class(class_text_input <> " px-4"),
        ]),
      ]),
    ]),
    render_slug_field(form),
    render_category_dual_pane_field(form, categories),
    render_body_field(form),
    render_locale_field(form),
    render_author_field(form, users),
    render_is_published_field(form),
    h.div([a.class(class_row)], [
      h.label([a.class(class_label)], [h.text("OpenGraph image")]),
      h.div([a.class(class_input_col)], [
        h.input([
          a.name("og_image"),
          a.type_("url"),
          a.value(formlib.field_value(form, "og_image")),
          a.class(class_text_input <> " px-4"),
        ]),
      ]),
    ]),
    h.hr([a.class("my-4")]),
    render_bottom_buttons(loading_status),
  ]

  let handle_submit = updates.process_post_form_data_to_produce_msg(
    _,
    form,
    False,
  )

  h.form(
    [
      a.method("post"),
      a.class("max-w-3xl mx-auto mb-8"),
      ev.on_submit(handle_submit),
    ],
    children,
  )
}

fn render_slug_field(form: Form(o)) {
  let handler_slug_click = {
    use elm <- decode.field("target", decode.dynamic)
    let editing_title = case br_element.cast(elm) {
      Ok(elm) -> {
        ffi.get_form_field_value(elm, "title")
      }
      Error(_e) -> Error(Nil)
    }
    editing_title
    |> result.map(fn(s) { decode.success(SlugGeneratorClicked(s)) })
    |> result.lazy_unwrap(fn() {
      decode.failure(SlugGeneratorClicked(""), "HTMLElement")
    })
  }
  h.div([a.class(class_row)], [
    h.label([a.class(class_label)], [h.text("Slug")]),
    h.div([a.class(class_input_col), a.class("relative")], [
      h.button(
        [
          a.class("absolute end-1 top-3 cursor-pointer"),
          a.type_("button"),
          ev.on("click", handler_slug_click),
        ],
        [
          lc_icons.refresh_ccw([
            a.class(
              "w-5 h-5 text-gray-400 transition-colors duration-300 dark:text-gray-500 hover:text-gray-500 dark:hover:text-gray-400",
            ),
          ]),
        ],
      ),
      h.input([
        a.name("slug"),
        a.type_("text"),
        a.required(True),
        a.value(formlib.field_value(form, "slug")),
        a.class(class_text_input <> " ps-4 pe-10"),
      ]),
    ]),
  ])
}

fn render_category_dual_pane_field(
  form: Form(PostEditablePart),
  categories: List(Category),
) {
  let selected_values = formlib.field_values(form, "categories")
  let #(selected_categories, available_categories) =
    categories
    |> list.partition(fn(c) { list.contains(selected_values, c.id) })
  h.div([a.class(class_row)], [
    h.label([a.class(class_label)], [h.text("Categories")]),
    h.div([a.class(class_input_col <> " grid grid-cols-2 gap-4 text-sm")], [
      h.div(
        [
          a.class(class_pane_in_field <> " min-h-20"),
        ],
        [
          h.label([a.class(class_pane_header)], [
            h.text("Available"),
          ]),
          h.div([a.class("py-2 ps-2")], [
            h.ol(
              [a.class("overflow-y-auto max-h-20 pe-3")],
              available_categories |> list.map(category_as_choice(_, False)),
            ),
          ]),
        ],
      ),
      h.div(
        [
          a.class(class_pane_in_field),
        ],
        [
          h.label([a.class(class_pane_header)], [h.text("Selected")]),
          h.div([a.class("py-2 ps-2")], [
            h.ol(
              [a.class("overflow-y-auto max-h-20 pe-3")],
              selected_categories |> list.map(category_as_choice(_, True)),
            ),
          ]),
        ],
      ),
    ]),
  ])
}

fn category_as_choice(category: Category, selected: Bool) {
  let icon = case selected {
    True -> lc_icons.arrow_left_from_line([a.class("h-5 w-5")])
    _ -> lc_icons.arrow_right_from_line([a.class("h-5 w-5")])
  }
  h.li([], [
    h.button(
      [
        a.type_("button"),
        a.class(
          "w-full cursor-pointer rounded py-0.5 px-1 hover:bg-gray-100 dark:hover:bg-gray-600 dark:hover:text-white flex  justify-between",
        ),
        a.classes([#("flex-row-reverse", selected)]),
        ev.on_click(UserMovedCategoryBetweenPane(category.id, !selected)),
      ],
      [h.span([], [h.text(category.title)]), icon],
    ),
  ])
}

fn render_body_field(form: Form(PostEditablePart)) -> Element(Msg(a)) {
  let handler_preview_click = {
    use elm <- decode.field("target", decode.dynamic)
    let editing_body = case br_element.cast(elm) {
      Ok(elm) -> {
        ffi.get_form_field_value(elm, "body")
      }
      Error(_e) -> Error(Nil)
    }
    editing_body
    |> result.map(fn(s) { decode.success(UserClickMarkdownPreview(s)) })
    |> result.lazy_unwrap(fn() {
      decode.failure(UserClickMarkdownPreview(""), "HTMLElement")
    })
  }
  h.div([a.class("mt-2 space-y-2")], [
    h.div([a.class("flex justify-between")], [
      h.label([a.class(class_label)], [h.text("Body")]),
      h.button(
        [
          a.type_("button"),
          a.class(
            "px-4 py-1.5 text-sm font-medium rounded-md text-gray-600 transition-colors duration-200 sm:text-base dark:hover:bg-gray-800 dark:text-gray-300 hover:bg-gray-100 border border-gray-400 dark:border-gray-700 cursor-pointer",
          ),
          ev.on("click", handler_preview_click),
        ],
        [h.text("Preview")],
      ),
    ]),
    h.div(
      [
        a.class(
          "border border-gray-300 dark:border-gray-600 rounded-md font-mono py-4",
        ),
      ],
      [
        h.div([a.class("px-2 h-80")], [
          h.textarea(
            [
              a.name("body"),
              a.type_("text"),
              a.class("w-full h-full focus:outline-none"),
            ],
            formlib.field_value(form, "body"),
          ),
        ]),
      ],
    ),
  ])
}

fn render_locale_field(form: Form(PostEditablePart)) {
  let choices = [#("en", "English"), #("vi", "Tiếng Việt")]

  h.div([a.class(class_row)], [
    h.label([a.class(class_label)], [h.text("Locale")]),
    h.div([], [
      widgets.render_single_select(
        "locale",
        choices,
        Some(formlib.field_value(form, "locale")),
        Some("Choose locale..."),
      ),
    ]),
  ])
}

fn render_author_field(form: Form(PostEditablePart), users: List(MiniUser)) {
  let choices = users |> list.map(fn(u) { #(u.id, u.email) })

  h.div([a.class(class_row)], [
    h.label([a.class(class_label)], [h.text("Author")]),
    h.div([], [
      widgets.render_single_select(
        "author",
        choices,
        Some(formlib.field_value(form, "author")),
        Some("Choose author..."),
      ),
    ]),
  ])
}

// Render "is_published" field with checkbox.
// Currently, with lustre + formal, user clicking to the checkbox
// doesn't make change to the value that "formal" manages.
// Then, when the form is submitted, the submitted value for checkbox field
// is still the one before user clicked.
// So we need to use "on_check" event handler to make form data in sync with what users see.
fn render_is_published_field(form: Form(PostEditablePart)) {
  let is_published = case formlib.field_value(form, "is_published") {
    "" -> False
    _ -> True
  }
  h.div([a.class(class_row)], [
    h.label([a.class(class_label)], [h.text("Published")]),
    h.div([a.class("pt-2")], [
      h.input([
        a.name("is_published"),
        a.type_("checkbox"),
        a.default_checked(is_published),
      ]),
    ]),
  ])
}

fn render_bottom_buttons(loading_status: LoadingStatus) -> Element(Msg(a)) {
  let submit_stay_click_handler =
    ev.on("click", {
      use elm <- decode.field("target", decode.dynamic)
      case br_element.cast(elm) {
        Ok(button) -> decode.success(SubmitStayButtonClicked(button))
        Error(_e) ->
          decode.failure(
            SubmitStayButtonClicked(document.body()),
            "HTMLButtonElement",
          )
      }
    })

  h.div([a.class("flex flex-row space-x-4")], [
    widgets.auto_submit_button(
      core.Sky,
      "Save and finish",
      loading_status == IsSubmitting,
    ),
    widgets.manual_submit_button(
      core.Purple,
      "Save and stay",
      submit_stay_click_handler,
      loading_status == IsSubmitting,
    ),
    h.div([a.class("grow")], []),
    h.button(
      [
        a.type_("reset"),
        a.class(
          "px-4 py-1.5 text-sm font-medium rounded-md text-gray-600 transition-colors duration-200 sm:text-base dark:hover:bg-gray-800 dark:text-gray-300 hover:bg-gray-100 border border-gray-400 dark:border-gray-700 cursor-pointer",
        ),
      ],
      [h.text("Reset")],
    ),
  ])
}

pub fn render_category_form(
  _cid: Option(String),
  form: Form(CategoryEditablePart),
  loading_status: LoadingStatus,
) {
  let children = [
    h.div([a.class(class_row)], [
      h.label([a.class(class_label)], [h.text("Title")]),
      h.div([a.class(class_input_col)], [
        h.input([
          a.name("title"),
          a.type_("text"),
          a.value(formlib.field_value(form, "title")),
          a.class(class_text_input <> " px-4"),
        ]),
      ]),
    ]),
    render_slug_field(form),
    h.div([a.class(class_row)], [
      h.label([a.class(class_label)], [h.text("Vietnamese title")]),
      h.div([a.class(class_input_col)], [
        h.input([
          a.name("title_vi"),
          a.type_("text"),
          a.value(formlib.field_value(form, "title_vi")),
          a.class(class_text_input <> " px-4"),
        ]),
      ]),
    ]),
    h.hr([a.class("p-4 border-b border-t-0")]),
    render_category_form_buttons(loading_status),
  ]

  let handle_submit = fn(submitted_values) {
    form
    |> formlib.set_values(submitted_values)
    |> formlib.run
    |> CategoryFormSubmitted
  }
  h.form(
    [
      a.method("post"),
      a.class("space-y-4 sm:space-y-0"),
      ev.on_submit(handle_submit),
    ],
    children,
  )
}

fn render_category_form_buttons(loading_status: LoadingStatus) {
  h.div([a.class("flex flex-row justify-between w-60 mx-auto sm:mt-4")], [
    widgets.auto_submit_button(core.Sky, "Save", loading_status == IsSubmitting),
    h.button(
      [
        a.type_("reset"),
        a.class(
          "px-4 py-1.5 text-sm font-medium rounded-md text-gray-600 transition-colors duration-200 sm:text-base dark:hover:bg-gray-800 dark:text-gray-300 hover:bg-gray-100 border border-gray-400 dark:border-gray-700 cursor-pointer",
        ),
        ev.on_click(FormCancelClicked),
      ],
      [h.text("Cancel")],
    ),
  ])
}
