import formal/form.{type Form} as formlib
import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}

import core.{
  type Book, type BookEditablePart, type Category, type CategoryEditablePart,
  type LoginData, type Post, type PostEditablePart, type Presentation,
  type PresentationEditablePart, BookEditablePart, CategoryEditablePart,
  LoginData, PostEditablePart, PresentationEditablePart,
}

pub fn create_login_form() -> Form(LoginData) {
  formlib.new({
    use email <- formlib.field("email", formlib.parse_string |> formlib.check_not_empty)
    use password <- formlib.field(
      "password",
      formlib.parse_string |> formlib.check_not_empty,
    )
    formlib.success(LoginData(email:, password:))
  })
}

pub fn make_post_form(post: Option(Post)) -> Form(PostEditablePart) {
  let form =
    formlib.new({
      use title <- formlib.field(
        "title",
        formlib.parse_string |> formlib.check_not_empty,
      )
      use slug <- formlib.field("slug", formlib.parse_string |> formlib.check_not_empty)
      use categories <- formlib.field(
        "categories",
        formlib.parse_list(formlib.parse_string),
      )
      use body <- formlib.field("body", formlib.parse_optional(formlib.parse_string))
      use locale <- formlib.field("locale", formlib.parse_optional(formlib.parse_string))
      use author <- formlib.field("author", formlib.parse_string)
      // This field is rendered as checkbox, which has special behaviour
      use is_published <- formlib.field("is_published", formlib.parse_checkbox)
      use og_image <- formlib.field(
        "og_image",
        formlib.parse_optional(formlib.parse_string),
      )
      formlib.success(PostEditablePart(
        title:,
        slug:,
        categories:,
        body:,
        locale:,
        author:,
        is_published:,
        og_image:,
      ))
    })
  case post {
    Some(p) -> {
      let serialized_categories =
        p.categories |> list.map(fn(c) { #("categories", c.id) })
      let initial = [
        #("title", p.title),
        #("slug", p.slug),
        #("author", p.author |> option.map(fn(u) { u.id }) |> option.unwrap("")),
      ]
      // These fields are Option
      let extra =
        [
          p.locale |> option.map(fn(s) { #("locale", s) }),
          p.body |> option.map(fn(s) { #("body", s) }),
          // This field is a checkbox, so we must not add value to it if we want it to represent "False".
          case p.is_published {
            True -> Some(#("is_published", "on"))
            _ -> None
          },
          p.og_image |> option.map(fn(s) { #("og_image", s) }),
        ]
        |> option.values

      let initial =
        initial
        |> list.append(extra)
        |> list.append(serialized_categories)
      formlib.add_values(form, initial)
    }
    _ -> {
      form
    }
  }
}

pub fn make_category_form(
  category: Option(Category),
) -> Form(CategoryEditablePart) {
  let form =
    formlib.new({
      use title <- formlib.field(
        "title",
        formlib.parse_string |> formlib.check_not_empty,
      )
      use slug <- formlib.field("slug", formlib.parse_string |> formlib.check_not_empty)
      use title_vi <- formlib.field(
        "title_vi",
        formlib.parse_optional(formlib.parse_string),
      )
      use header_color <- formlib.field(
        "header_color",
        formlib.parse_optional(formlib.parse_string),
      )
      use summary_en <- formlib.field(
        "summary_en",
        formlib.parse_optional(formlib.parse_string),
      )
      use summary_vi <- formlib.field(
        "summary_vi",
        formlib.parse_optional(formlib.parse_string),
      )
      use featured_order <- formlib.field(
        "featured_order",
        formlib.parse_optional(formlib.parse_int),
      )
      formlib.success(CategoryEditablePart(
        title:,
        slug:,
        title_vi:,
        header_color:,
        featured_order:,
        summary_en:,
        summary_vi:,
      ))
    })
  case category {
    Some(c) -> {
      let initial = [
        #("title", c.title),
        #("slug", c.slug),
        #("title_vi", c.title_vi |> option.unwrap("")),
        #("header_color", c.header_color |> option.unwrap("")),
        #("summary_en", c.summary_en |> option.unwrap("")),
        #("summary_vi", c.summary_vi |> option.unwrap("")),
        #("featured_order", c.featured_order |> option.map(int.to_string) |> option.unwrap("")),
      ]
      formlib.add_values(form, initial)
    }
    _ -> form
  }
}

pub fn make_presentation_form(
  presentation: Option(Presentation),
) -> Form(PresentationEditablePart) {
  let form =
    formlib.new({
      use title <- formlib.field(
        "title",
        formlib.parse_string |> formlib.check_not_empty,
      )
      use url <- formlib.field("url", formlib.parse_string |> formlib.check_not_empty)
      use event <- formlib.field("event", formlib.parse_optional(formlib.parse_string))
      formlib.success(PresentationEditablePart(title:, url:, event:))
    })
  case presentation {
    Some(p) -> {
      let initial = [
        #("title", p.title),
        #("url", p.url),
        #("event", p.event |> option.unwrap("")),
      ]
      formlib.add_values(form, initial)
    }
    _ -> form
  }
}

pub fn make_book_form(book: Option(Book)) -> Form(BookEditablePart) {
  let form =
    formlib.new({
      use title <- formlib.field(
        "title",
        formlib.parse_string |> formlib.check_not_empty,
      )
      use download_url <- formlib.field(
        "download_url",
        formlib.parse_optional(formlib.parse_string),
      )
      use author_id <- formlib.field(
        "author_id",
        formlib.parse_optional(formlib.parse_string),
      )
      formlib.success(BookEditablePart(title:, download_url:, author_id:))
    })
  case book {
    Some(b) -> {
      let initial = [
        #("title", b.title),
        #("download_url", b.download_url |> option.unwrap("")),
        #(
          "author_id",
          b.author |> option.map(fn(a) { a.id }) |> option.unwrap(""),
        ),
      ]
      formlib.add_values(form, initial)
    }
    _ -> form
  }
}
