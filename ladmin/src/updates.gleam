import formal/form as formlib
import forms
import gleam/dynamic/decode
import gleam/http/response.{Response}
import gleam/io
import gleam/javascript/array
import gleam/json
import gleam/list
import gleam/option.{None, Some}
import gleam/pair
import gleam/result
import lustre/effect.{type Effect}
import plinth/browser/element.{type Element}
import plinth/javascript/storage
import rsvp

import actions
import consts
import core.{
  type ApiListingResponse, type Category, type LoginData, type MiniPost,
  type Msg, type Post, type PostEditablePart, type User, ApiListingResponse,
  CheckBoxes, LoggedIn, NonLogin, PageOwnedCategories, PageOwnedObjectPaging,
  PageOwnedPosts, PostFormSubmitted, TryingLogin,
}
import decoders.{encode_user}
import ffi
import models.{type AppMsg, type Model, Model}
import routes.{
  type Route, CategoryEditPage, CategoryListPage, HomePage, LoginPage,
  PostEditPage, PostListPage,
}

pub type LoginValidationDetail {
  LoginFailureDetail(message: String, email: String, password: String)
}

pub fn handle_router_init_done(model: Model) {
  io.println("RouterInitDone")
  let Model(
    route:,
    login_state:,
    mounted_path:,
    categories:,
    partial_load_categories:,
    ..,
  ) = model
  echo route

  let #(whatsnext, is_loading) = case route, login_state {
    LoginPage, _ -> #(effect.none(), False)
    // If user has already logged-in, and visiting HomePage, redirect to PostList
    HomePage, LoggedIn(_u) -> {
      #(routes.goto(PostListPage(None, None, None), mounted_path), False)
    }
    // In PostList page, call API to load posts
    PostListPage(Some(p), _q, _c), _ if p < 1 -> #(
      routes.goto(PostListPage(None, None, None), mounted_path),
      False,
    )
    PostListPage(p, q, cat_id), LoggedIn(_u) -> {
      let load_posts_action = actions.load_posts(option.unwrap(p, 1), q, cat_id)
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(effect.batch([load_posts_action, load_categories_action]), True)
    }
    PostEditPage(id), _ -> {
      let #(load_post_action, is_loading) = case id {
        "" -> #(effect.none(), False)
        s -> #(actions.load_single_post(s), True)
      }
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(
        effect.batch([
          load_post_action,
          load_categories_action,
          actions.load_users(),
        ]),
        is_loading,
      )
    }
    // In CategoryListPagei page, call API to load categories
    CategoryListPage(Some(p)), _ if p < 1 -> {
      #(routes.goto(CategoryListPage(None), mounted_path), False)
    }
    CategoryListPage(p), _ -> {
      #(actions.load_categories(option.unwrap(p, 1)), True)
    }
    CategoryEditPage(id), _ -> {
      #(actions.load_single_category(id), True)
    }
    // Already logged in, just serve, no redirect
    _, LoggedIn(_u) -> #(effect.none(), False)
    _, _ -> {
      #(routes.goto(LoginPage, mounted_path), False)
    }
  }
  // If the initial page is the "create post" page, create a form
  let post_form = case route {
    PostEditPage("") -> Some(forms.make_post_form(None))
    _ -> model.post_form
  }
  let model = Model(..model, is_loading:, post_form:)
  #(model, whatsnext)
}

pub fn handle_login_submission(
  model: Model,
  form: Result(LoginData, formlib.Form(LoginData)),
) -> #(Model, Effect(AppMsg)) {
  case form {
    Ok(login_data) -> {
      io.println("Form valid")
      // Form is validated, call API
      #(model, actions.login_via_api(login_data))
    }
    Error(form) -> {
      io.println("Form invalid")
      echo formlib.all_errors(form)
      let model = Model(..model, login_state: TryingLogin(form))
      #(model, effect.none())
    }
  }
}

pub fn handle_login_api_result(model: Model, res: Result(User, rsvp.Error)) {
  case res {
    Ok(user) -> {
      let login_state = LoggedIn(user)
      // User has logged-in successfully. Redirect to home page
      let go_next = routes.goto(HomePage, model.mounted_path)
      let model = Model(..model, login_state:)
      // Save to localstorage
      storage.local()
      |> result.try(storage.set_item(
        _,
        consts.key_store_user,
        json.to_string(encode_user(user)),
      ))
      |> result.lazy_unwrap(fn() {
        io.println_error("Failed to acquire localStorage!")
      })
      let flash_messages = [
        models.create_success_message("Login successfully!"),
        ..model.flash_messages
      ]
      let model = Model(..model, flash_messages:)
      let schedule = models.schedule_cleaning_flash_messages()
      #(model, effect.batch([go_next, schedule]))
    }
    Error(err) -> {
      let detail = case err {
        rsvp.HttpError(Response(body:, status:, ..)) if status == 422 -> {
          let fields_decoder = {
            use email <- decode.field("email", decode.string)
            use password <- decode.field("password", decode.string)
            decode.success(#(email, password))
          }
          let validation_error_decoder = {
            use fields <- decode.field("fields", fields_decoder)
            use message <- decode.field("message", decode.string)
            decode.success(LoginFailureDetail(message, fields.0, fields.1))
          }
          json.parse(body, validation_error_decoder)
          |> result.unwrap(LoginFailureDetail("Some error", "", ""))
        }
        _ -> LoginFailureDetail("Some error", "", "")
      }
      let login_state = case model.login_state {
        TryingLogin(form) -> {
          let form =
            form
            |> formlib.add_error("email", formlib.CustomError(detail.email))
            |> formlib.add_error(
              "password",
              formlib.CustomError(detail.password),
            )
          TryingLogin(form)
        }
        s -> s
      }
      let model = Model(..model, login_state:)
      #(model, effect.none())
    }
  }
}

pub fn handle_api_list_post_result(
  model: Model,
  res: Result(ApiListingResponse(MiniPost), rsvp.Error),
) -> Model {
  case res {
    Ok(info) -> {
      let ApiListingResponse(count:, total_pages:, links:, ..) = info
      Model(
        ..model,
        page_owned_objects: core.PageOwnedPosts(info.objects),
        page_owned_object_paging: PageOwnedObjectPaging(
          count:,
          total_pages:,
          links:,
        ),
        is_loading: False,
      )
    }
    Error(e) -> {
      io.println_error("Posts API failed")
      echo e
      let message = models.create_danger_message("Failed to load posts")
      let Model(flash_messages:, ..) = model
      Model(
        ..model,
        flash_messages: [message, ..flash_messages],
        is_loading: False,
      )
    }
  }
}

pub fn handle_successful_logout(model: Model) -> #(Model, Effect(Msg(a))) {
  let login_state = NonLogin
  // Delete user from localStorage
  storage.local()
  |> result.map(storage.remove_item(_, consts.key_store_user))
  |> result.map(fn(_x) {
    io.println("Deleted " <> consts.key_store_user <> " from localStorage!")
  })
  |> result.unwrap(Nil)
  let flash_messages = [
    models.create_info_message("Logged out successfully."),
    ..model.flash_messages
  ]
  let model = Model(..model, login_state:, flash_messages:)
  #(model, models.schedule_cleaning_flash_messages())
}

pub fn handle_landing_on_page(new_route: Route, model: Model) {
  let Model(mounted_path:, categories:, partial_load_categories:, ..) = model
  let login_state = case new_route, model.login_state {
    LoginPage, NonLogin -> TryingLogin(forms.create_login_form())
    _, state -> state
  }
  let #(go_next, is_loading) = case new_route, login_state {
    // If user has logged-in, redirect to "/posts" page
    HomePage, LoggedIn(_u) -> {
      #(routes.goto(PostListPage(None, None, None), mounted_path), False)
    }
    PostListPage(p, q, cat_id), _ -> {
      let load_posts_action = actions.load_posts(option.unwrap(p, 1), q, cat_id)
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(effect.batch([load_posts_action, load_categories_action]), True)
    }
    PostEditPage(id), _ -> {
      let #(load_post_action, is_loading) = case id {
        "" -> #(effect.none(), False)
        s -> #(actions.load_single_post(s), True)
      }
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(
        effect.batch([
          load_post_action,
          load_categories_action,
          actions.load_users(),
        ]),
        is_loading,
      )
    }
    CategoryListPage(p), _ -> {
      let load_categories_action = actions.load_categories(option.unwrap(p, 1))
      #(load_categories_action, True)
    }
    _, _ -> #(effect.none(), False)
  }
  io.println("Reset page_owned_objects")
  echo new_route
  let #(post_form, page_owned_objects) = case new_route {
    PostEditPage("") -> #(Some(forms.make_post_form(None)), PageOwnedPosts([]))
    PostEditPage(_id) -> #(model.post_form, PageOwnedPosts([]))
    _ -> #(None, model.page_owned_objects)
  }
  let model =
    Model(
      ..model,
      route: new_route,
      login_state:,
      is_loading:,
      post_form:,
      page_owned_objects:,
    )
  #(model, go_next)
}

pub fn handle_api_list_category_result(
  model: Model,
  res: Result(ApiListingResponse(Category), rsvp.Error),
) {
  let Model(route:, ..) = model
  case res {
    Error(e) -> {
      io.println_error("Categories API failed")
      echo e
      let message = models.create_danger_message("Failed to load posts")
      let Model(flash_messages:, ..) = model
      let model =
        Model(
          ..model,
          flash_messages: [message, ..flash_messages],
          is_loading: False,
        )
      #(model, effect.none())
    }
    Ok(info) -> {
      let ApiListingResponse(count:, total_pages:, links:, ..) = info
      let Model(partial_load_categories:, ..) = model
      case route {
        // This page, we need to load all categories from API
        PostListPage(_x, _q, _c) | PostEditPage(_id) -> {
          let categories = list.append(partial_load_categories, info.objects)
          let model = Model(..model, partial_load_categories: categories)
          let #(model, whatsnext) = case links.1 {
            Some(u) -> {
              #(
                Model(..model, partial_load_categories: categories),
                actions.load_categories_by_url(u),
              )
            }
            None -> {
              #(
                Model(..model, categories:, partial_load_categories: []),
                effect.none(),
              )
            }
          }
          #(model, whatsnext)
        }
        CategoryListPage(_p) -> {
          let model =
            Model(
              ..model,
              page_owned_objects: PageOwnedCategories(info.objects),
              page_owned_object_paging: PageOwnedObjectPaging(
                count:,
                total_pages:,
                links:,
              ),
              is_loading: False,
            )
          #(model, effect.none())
        }
        _ -> {
          #(model, effect.none())
        }
      }
    }
  }
}

pub fn handle_api_retrieve_post_result(
  model: Model,
  res: Result(Post, rsvp.Error),
) {
  case res {
    Ok(p) -> {
      let form = forms.make_post_form(Some(p))
      let model =
        Model(
          ..model,
          post_form: Some(form),
          checkboxes: CheckBoxes(is_published: p.is_published),
          is_loading: False,
        )
      #(model, effect.none())
    }
    Error(_e) -> {
      let message = models.create_danger_message("Failed to load post")
      let Model(flash_messages:, ..) = model
      let model =
        Model(
          ..model,
          flash_messages: [message, ..flash_messages],
          is_loading: False,
        )
      #(model, effect.none())
    }
  }
}

pub fn handle_api_slug_generation(
  model: Model,
  res: Result(String, rsvp.Error),
) -> Model {
  case res {
    Error(_e) -> model
    Ok(slug) -> {
      let #(post_form, category_form) = case
        model.post_form,
        model.category_form
      {
        Some(form), None -> #(
          Some(formlib.add_string(form, "slug", slug)),
          None,
        )
        None, Some(form) -> #(
          None,
          Some(formlib.add_string(form, "slug", slug)),
        )
        p, q -> #(p, q)
      }
      Model(..model, post_form:, category_form:)
    }
  }
}

pub fn handle_post_form_submission(
  model: Model,
  res: Result(PostEditablePart, formlib.Form(PostEditablePart)),
  stay: Bool,
) {
  case res {
    Ok(data) -> {
      let whatsnext = case model.route {
        PostEditPage("") -> actions.create_post_via_api(data)
        PostEditPage(id) -> actions.update_post_via_api(id, data, stay)
        _ -> effect.none()
      }
      #(model, whatsnext)
    }
    Error(form) -> {
      let post_form = model.post_form |> option.map(fn(_f) { form })
      #(Model(..model, post_form:), effect.none())
    }
  }
}

pub fn handle_api_update_post_result(
  model: Model,
  res: Result(Post, rsvp.Error),
  stay: Bool,
) {
  case res {
    Error(_e) -> {
      let message = models.create_danger_message("Failed to save post.")
      let flash_messages = [message, ..model.flash_messages]
      #(
        Model(..model, flash_messages:),
        models.schedule_cleaning_flash_messages(),
      )
    }
    Ok(post) -> {
      let message =
        models.create_success_message(
          "Post " <> post.title <> " has been updated.",
        )
      let flash_messages = [message, ..model.flash_messages]
      let gonext = case stay {
        False -> routes.goto(PostListPage(None, None, None), model.mounted_path)
        True -> effect.none()
      }
      #(
        Model(..model, flash_messages:),
        effect.batch([
          models.schedule_cleaning_flash_messages(),
          gonext,
        ]),
      )
    }
  }
}

// Handle the case that a Post has just been created.
// We will redirect user to the edit page.
pub fn handle_api_create_post_result(
  model: Model,
  res: Result(Post, rsvp.Error),
) {
  case res {
    Error(_e) -> {
      let message = models.create_danger_message("Failed to save post.")
      let flash_messages = [message, ..model.flash_messages]
      #(Model(..model, flash_messages:), effect.none())
    }
    Ok(post) -> {
      let message =
        models.create_success_message(
          "Post " <> post.title <> " has been created.",
        )
      let flash_messages = [message, ..model.flash_messages]
      let whatsnext =
        effect.batch([
          routes.goto(PostEditPage(post.id), model.mounted_path),
          models.schedule_cleaning_flash_messages(),
        ])
      #(Model(..model, flash_messages:), whatsnext)
    }
  }
}

pub fn handle_category_moved_between_panes(
  model: Model,
  id: String,
  to_move_in: Bool,
) -> Model {
  let post_form =
    model.post_form
    |> option.map(push_in_or_out_category_from_form(_, id, to_move_in))
  Model(..model, post_form:)
}

fn push_in_or_out_category_from_form(
  form: formlib.Form(PostEditablePart),
  value: String,
  to_move_in: Bool,
) {
  let values_in_form = formlib.field_values(form, "categories")
  case to_move_in, list.contains(values_in_form, value) {
    True, False -> [value, ..values_in_form]
    False, True -> values_in_form |> list.filter(fn(v) { v != value })
    _, _ -> values_in_form
  }
  |> list.map(fn(v) { #("categories", v) })
  |> formlib.add_values(form, _)
}

pub fn handle_rendered_markdown_received(model: Model, html: String) {
  let model = Model(..model, post_body_preview: Some(html))
  let whatsnext = {
    use _dispatch, _root <- effect.after_paint
    ffi.show_dialog("." <> consts.selector_post_body_preview_dialog)
    Nil
  }
  #(model, whatsnext)
}

pub fn handle_submit_stay_button_clicked(
  model: Model,
  button: Element,
) -> #(Model, Effect(Msg(a))) {
  let whatsnext = case model.post_form {
    Some(form) -> {
      let whatsnext = {
        use dispatch, _root <- effect.after_paint
        let app_msg =
          element.closest(button, "form")
          |> result.map(ffi.get_form_data)
          |> result.map(array.to_list)
          |> result.map(process_post_form_data_to_produce_msg(_, form, True))
        app_msg |> result.map(dispatch) |> result.unwrap(Nil)
      }
      whatsnext
    }
    None -> effect.none()
  }
  #(model, whatsnext)
}

fn process_post_form_data_to_produce_msg(
  submitted_values: List(#(String, String)),
  form: formlib.Form(PostEditablePart),
  stay: Bool,
) {
  // If the checkbox is unchecked, the "is_published" field will not be in submitted data.
  // When the checkbox value is missing, we should clear its previous data from the "formal" form.
  // formal doesn't provide a function like remove_value, so we have to use set_values.
  let multi_value_field = "categories"
  let new_values =
    formlib.field_values(form, multi_value_field)
    |> list.map(pair.new(multi_value_field, _))
    |> list.append(submitted_values, _)
  form
  |> formlib.set_values(new_values)
  |> formlib.run
  |> PostFormSubmitted(stay)
}

pub fn handle_api_retrieve_category_result(
  model: Model,
  res: Result(Category, rsvp.Error),
) {
  case res {
    Ok(cat) -> {
      let form = forms.make_category_form(Some(cat))
      let model = Model(..model, category_form: Some(form), is_loading: False)
      #(model, effect.none())
    }
    Error(_e) -> {
      let message = models.create_danger_message("Failed to load category")
      let Model(flash_messages:, ..) = model
      let model =
        Model(
          ..model,
          flash_messages: [message, ..flash_messages],
          is_loading: False,
        )
      #(model, effect.none())
    }
  }
}
