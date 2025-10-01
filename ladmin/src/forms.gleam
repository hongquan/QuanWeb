import formal/form.{type Form}
import gleam/list
import gleam/option.{type Option, Some}

import core.{type LoginData, type Post, LoginData, PostEditablePart}

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

pub fn make_post_form(post: Option(Post)) -> Form(core.PostEditablePart) {
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
      form.success(PostEditablePart(title:, slug:, categories:))
    })
  case post {
    Some(p) -> {
      let serialized_categories =
        p.categories |> list.map(fn(c) { #("categories", c.id) })
      let initial =
        [#("title", p.title), #("slug", p.slug)]
        |> list.append(serialized_categories)
      form.add_values(form, initial)
    }
    _ -> {
      form
    }
  }
}
