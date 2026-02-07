import formal/form.{type Form}
import gleam/int
import gleam/list
import gleam/option.{type Option, None, Some}

import core.{
  type Category, type CategoryEditablePart, type LoginData, type Post,
  type PostEditablePart, CategoryEditablePart, LoginData, PostEditablePart,
}

pub fn create_login_form() -> Form(LoginData) {
  form.new({
    use email <- form.field("email", form.parse_string |> form.check_not_empty)
    use password <- form.field(
      "password",
      form.parse_string |> form.check_not_empty,
    )
    form.success(LoginData(email:, password:))
  })
}

pub fn make_post_form(post: Option(Post)) -> Form(PostEditablePart) {
  let form =
    form.new({
      use title <- form.field(
        "title",
        form.parse_string |> form.check_not_empty,
      )
      use slug <- form.field("slug", form.parse_string |> form.check_not_empty)
      use categories <- form.field(
        "categories",
        form.parse_list(form.parse_string),
      )
      use body <- form.field("body", form.parse_optional(form.parse_string))
      use locale <- form.field("locale", form.parse_optional(form.parse_string))
      use author <- form.field("author", form.parse_string)
      // This field is rendered as checkbox, which has special behaviour
      use is_published <- form.field("is_published", form.parse_checkbox)
      use og_image <- form.field(
        "og_image",
        form.parse_optional(form.parse_string),
      )
      form.success(PostEditablePart(
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
      form.add_values(form, initial)
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
    form.new({
      use title <- form.field(
        "title",
        form.parse_string |> form.check_not_empty,
      )
      use slug <- form.field("slug", form.parse_string |> form.check_not_empty)
      use title_vi <- form.field(
        "title_vi",
        form.parse_optional(form.parse_string),
      )
      use header_color <- form.field(
        "header_color",
        form.parse_optional(form.parse_string),
      )
      use summary_en <- form.field(
        "summary_en",
        form.parse_optional(form.parse_string),
      )
      use summary_vi <- form.field(
        "summary_vi",
        form.parse_optional(form.parse_string),
      )
      use featured_order <- form.field(
        "featured_order",
        form.parse_optional(form.parse_int),
      )
      form.success(CategoryEditablePart(
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
      form.add_values(form, initial)
    }
    _ -> form
  }
}
