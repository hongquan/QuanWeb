import formal/form as formlib
import forms
import gleam/dynamic/decode
import gleam/http/response.{Response}
import gleam/io
import gleam/json
import gleam/list
import gleam/option.{None, Some}
import gleam/result
import lustre/effect.{type Effect}
import plinth/javascript/storage
import rsvp

import actions
import consts
import core.{
  type ApiListingResponse, type Category, type LoginData, type Msg, type Post,
  type PostEditablePart, type User, ApiListingResponse, LoggedIn, NonLogin,
  PageOwnedObjectPaging, PageOwnedPosts, PostCreating, PostEditing, TryingLogin,
}
import decoders.{encode_user}
import models.{type AppMsg, type Model, Model}
import routes.{
  type Route, CategoryListPage, HomePage, LoginPage, PostEditPage, PostListPage,
}

pub type LoginValidationDetail {
  LoginFailureDetail(message: String, email: String, password: String)
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
  res: Result(ApiListingResponse(Post), rsvp.Error),
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
      #(effect.batch([load_post_action, load_categories_action]), is_loading)
    }
    _, _ -> #(effect.none(), False)
  }
  let #(post_editing, page_owned_objects) = case new_route {
    PostEditPage("") -> #(
      core.PostCreating(forms.make_post_form(None)),
      PageOwnedPosts([]),
    )
    PostEditPage(_id) -> #(model.post_editing, PageOwnedPosts([]))
    _ -> #(core.NoPostEditing, model.page_owned_objects)
  }
  let model =
    Model(
      ..model,
      route: new_route,
      login_state:,
      is_loading:,
      post_editing:,
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
              page_owned_objects: core.PageOwnedCategories(info.objects),
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

pub fn handle_api_load_post_result(model: Model, res: Result(Post, rsvp.Error)) {
  case res {
    Ok(p) -> {
      let form = forms.make_post_form(Some(p))
      let model =
        Model(
          ..model,
          post_editing: core.PostEditing(p, form),
          is_loading: False,
        )
      #(model, effect.none())
    }
    Error(_e) -> {
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
  }
}

pub fn handle_api_slug_result(
  model: Model,
  res: Result(String, rsvp.Error),
) -> Model {
  case res {
    Error(_e) -> model
    Ok(s) -> {
      let post_editing = case model.post_editing {
        PostEditing(post:, form:) -> {
          PostEditing(post, formlib.add_string(form, "slug", s))
        }
        PostCreating(form) -> PostCreating(formlib.add_string(form, "slug", s))
        n -> n
      }
      Model(..model, post_editing:)
    }
  }
}

pub fn handle_post_form_submission(
  model: Model,
  res: Result(PostEditablePart, formlib.Form(PostEditablePart)),
) {
  case res {
    Ok(data) -> {
      let whatsnext = case model.post_editing {
        PostEditing(post:, ..) -> actions.update_post_via_api(post.id, data)
        PostCreating(_f) -> actions.create_post_via_api(data)
        _ -> effect.none()
      }
      #(model, whatsnext)
    }
    Error(form) -> {
      let post_editing = case model.post_editing {
        PostEditing(post:, ..) -> {
          PostEditing(post, form)
        }
        PostCreating(_f) -> PostCreating(form)
        n -> n
      }
      #(Model(..model, post_editing:), effect.none())
    }
  }
}

pub fn handle_api_update_post_result(
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
          "Post " <> post.title <> " has been updated.",
        )
      let flash_messages = [message, ..model.flash_messages]
      let post_editing = case model.post_editing {
        PostEditing(form:, ..) -> PostEditing(post, form)
        _ -> core.NoPostEditing
      }
      #(
        Model(..model, post_editing:, flash_messages:),
        models.schedule_cleaning_flash_messages(),
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
      let post_editing = case model.post_editing {
        PostCreating(form) -> PostEditing(post, form)
        _ -> core.NoPostEditing
      }
      let whatsnext =
        effect.batch([
          routes.goto(PostEditPage(post.id), model.mounted_path),
          models.schedule_cleaning_flash_messages(),
        ])
      #(Model(..model, post_editing:, flash_messages:), whatsnext)
    }
  }
}

pub fn handle_category_moved_between_panes(
  model: Model,
  id: String,
  to_move_in: Bool,
) -> Model {
  let post_editing = case model.post_editing {
    PostCreating(form) -> {
      push_in_or_out_category_from_form(form, id, to_move_in)
      |> PostCreating
    }
    PostEditing(p, form) -> {
      push_in_or_out_category_from_form(form, id, to_move_in)
      |> PostEditing(p, _)
    }
    n -> n
  }
  Model(..model, post_editing:)
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
