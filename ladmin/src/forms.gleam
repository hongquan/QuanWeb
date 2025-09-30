import formal/form.{type Form}
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
      form.success(PostEditablePart(title:, slug:))
    })
  case post {
    Some(p) -> {
      let initial = [#("title", p.title), #("slug", p.slug)]
      form.add_values(form, initial)
    }
    _ -> {
      form
    }
  }
}
