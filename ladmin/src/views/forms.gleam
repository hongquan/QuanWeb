import formal/form as formlib
import gleam/dynamic/decode
import gleam/option.{type Option}
import gleam/result
import lustre/attribute as a
import lustre/element/html as h
import lustre/event as ev
import plinth/browser/element as br_element

import core.{
  type Post, type PostEditablePart, PostFormSubmitted, SlugGeneratorClicked,
}
import ffi
import lucide_lustre as lucide_icon

const class_row = "sm:grid sm:grid-cols-4 sm:items-start sm:gap-2 sm:py-2"

const class_label = "block font-medium leading-6 dark:text-white sm:pt-2"

const class_input_col = "mt-2 sm:col-span-3 sm:mt-0"

const class_text_input = "py-2 w-full text-gray-700 bg-white border rounded-md sm:mx-2 dark:bg-gray-900 dark:text-gray-300 dark:border-gray-600 focus:border-blue-400 dark:focus:border-blue-300 focus:outline-none focus:ring focus:ring-blue-300 focus:ring-opacity-40"

pub fn render_post_form(
  _post: Option(Post),
  form: formlib.Form(PostEditablePart),
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
    h.hr([a.class("my-4")]),
    h.div([], [
      h.button(
        [
          a.type_("submit"),
          a.class(
            "px-8 py-2.5 leading-5 text-white transition-colors duration-300 transform bg-sky-700 rounded-md hover:bg-sky-600 focus:outline-none focus:bg-sky-600 cursor-pointer",
          ),
        ],
        [h.text("Save")],
      ),
    ]),
  ]
  let handle_submit = fn(values) {
    form |> formlib.add_values(values) |> formlib.run |> PostFormSubmitted
  }
  h.form(
    [
      a.method("post"),
      a.class("max-w-3xl mx-auto"),
      ev.on_submit(handle_submit),
    ],
    children,
  )
}

fn render_slug_field(form: formlib.Form(PostEditablePart)) {
  let handler_slug_click = {
    use elm <- decode.field("target", decode.dynamic)
    let editing_title = case br_element.cast(elm) {
      Ok(elm) -> {
        echo elm
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
          lucide_icon.refresh_ccw([
            a.class(
              "w-5 h-5 text-gray-400 transition-colors duration-300 dark:text-gray-500 hover:text-gray-500 dark:hover:text-gray-400",
            ),
          ]),
        ],
      ),
      h.input([
        a.name("slug"),
        a.type_("text"),
        a.value(formlib.field_value(form, "slug")),
        a.class(class_text_input <> " ps-4 pe-10"),
      ]),
    ]),
  ])
}
