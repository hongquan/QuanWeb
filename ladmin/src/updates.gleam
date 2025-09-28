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
  type ApiListingResponse, type Category, type LoginData, type Post, type User,
  ApiListingResponse, LoggedIn, NonLogin, PageOwnedObjectPaging, TryingLogin,
}
import decoders.{encode_user}
import models.{type AppMsg, type Model, Model}
import routes.{type Route, CategoryListPage, HomePage, LoginPage, PostListPage}

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

pub fn handle_login_api_result(
  model: Model,
  res: Result(User, rsvp.Error),
) -> #(Model, Effect(a)) {
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
      #(model, go_next)
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

pub fn handle_successful_logout(model: Model) -> Model {
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
  model
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
    PostListPage(p, _q, _c), _ -> {
      let load_posts_action = actions.load_posts(option.unwrap(p, 1))
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(effect.batch([load_posts_action, load_categories_action]), True)
    }
    _, _ -> #(effect.none(), False)
  }
  let model = Model(..model, route: new_route, login_state:, is_loading:)
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
        PostListPage(_x, _q, _c) -> {
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
