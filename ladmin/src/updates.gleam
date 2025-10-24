import formal/form.{type Form} as formlib
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
  type ApiListingResponse, type Category, type CategoryEditablePart,
  type ContentItemId, type LoginData, type MiniPost, type Msg, type Post,
  type PostEditablePart, type User, ApiListingResponse, CheckBoxes, IsLoading,
  IsSubmitting, LoggedIn, NonLogin, PageOwnedCategories, PageOwnedObjectPaging,
  PageOwnedPosts, PostFormSubmitted, PostId, TryingLogin,
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
  let Model(route:, login_state:, categories:, partial_load_categories:, ..) =
    model
  echo route

  let #(whatsnext, loading_status) = case route, login_state {
    LoginPage, _ -> #(effect.none(), core.Idle)
    // If user has already logged-in, and visiting HomePage, redirect to PostList
    HomePage, LoggedIn(_u) -> {
      #(routes.goto(PostListPage(None, None, None)), core.Idle)
    }
    // In PostList page, call API to load posts
    PostListPage(Some(p), _q, _c), _ if p < 1 -> #(
      routes.goto(PostListPage(None, None, None)),
      core.Idle,
    )
    PostListPage(p, q, cat_id), LoggedIn(_u) -> {
      let load_posts_action = actions.load_posts(option.unwrap(p, 1), q, cat_id)
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(effect.batch([load_posts_action, load_categories_action]), IsLoading)
    }
    PostEditPage(id), _ -> {
      let #(load_post_action, loading_status) = case id {
        "" -> #(effect.none(), core.Idle)
        s -> #(actions.load_single_post(s), IsLoading)
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
        loading_status,
      )
    }
    // In CategoryListPagei page, call API to load categories
    CategoryListPage(Some(p)), _ if p < 1 -> {
      #(routes.goto(CategoryListPage(None)), core.Idle)
    }
    CategoryListPage(p), _ -> {
      #(actions.load_categories(option.unwrap(p, 1)), IsLoading)
    }
    CategoryEditPage(id), _ if id != "" -> {
      #(actions.load_single_category(id), IsLoading)
    }
    // Already logged in, just serve, no redirect
    _, LoggedIn(_u) -> #(effect.none(), core.Idle)
    _, _ -> {
      #(routes.goto(LoginPage), core.Idle)
    }
  }
  // If the initial page is the "create post" page, create a form
  let post_form = case route {
    PostEditPage("") -> Some(forms.make_post_form(None))
    _ -> model.post_form
  }
  let category_form = case route {
    CategoryEditPage("") -> Some(forms.make_category_form(None))
    _ -> model.category_form
  }
  let model = Model(..model, loading_status:, post_form:, category_form:)
  #(model, whatsnext)
}

pub fn handle_login_submission(
  form: Result(LoginData, Form(LoginData)),
  model: Model,
) -> #(Model, Effect(AppMsg)) {
  case form {
    Ok(login_data) -> {
      let model = Model(..model, loading_status: IsSubmitting)
      // Form is validated, call API
      #(model, actions.login_via_api(login_data))
    }
    Error(form) -> {
      echo formlib.all_errors(form)
      let model = Model(..model, login_state: TryingLogin(form))
      #(model, effect.none())
    }
  }
}

pub fn handle_login_api_result(res: Result(User, rsvp.Error), model: Model) {
  // Reset loading status
  let model = Model(..model, loading_status: core.Idle)
  case res {
    Ok(user) -> {
      let login_state = LoggedIn(user)
      // User has logged-in successfully. Redirect to home page
      let go_next = routes.goto(HomePage)
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
  res: Result(ApiListingResponse(MiniPost), rsvp.Error),
  model: Model,
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
        loading_status: core.Idle,
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
        loading_status: core.Idle,
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
  let gonext =
    effect.batch([
      routes.goto(HomePage),
      models.schedule_cleaning_flash_messages(),
    ])
  #(model, gonext)
}

pub fn handle_landing_on_page(new_route: Route, model: Model) {
  let Model(categories:, partial_load_categories:, ..) = model
  let login_state = case new_route, model.login_state {
    LoginPage, NonLogin -> TryingLogin(forms.create_login_form())
    _, state -> state
  }
  let #(go_next, loading_status) = case new_route, login_state {
    // If user has logged-in, redirect to "/posts" page
    HomePage, LoggedIn(_u) -> {
      #(routes.goto(PostListPage(None, None, None)), core.Idle)
    }
    // If user has not logged-in, redirect to Login page
    _, NonLogin -> {
      #(routes.goto(LoginPage), core.Idle)
    }
    PostListPage(p, q, cat_id), _ -> {
      let load_posts_action = actions.load_posts(option.unwrap(p, 1), q, cat_id)
      let load_categories_action = case categories, partial_load_categories {
        [], _o -> actions.load_categories(1)
        _, _ -> effect.none()
      }
      #(effect.batch([load_posts_action, load_categories_action]), IsLoading)
    }
    PostEditPage(id), _ -> {
      let #(load_post_action, loading_status) = case id {
        "" -> #(effect.none(), core.Idle)
        s -> #(actions.load_single_post(s), IsLoading)
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
        loading_status,
      )
    }
    CategoryListPage(p), _ -> {
      let load_categories_action = actions.load_categories(option.unwrap(p, 1))
      #(load_categories_action, IsLoading)
    }
    CategoryEditPage(id), _ if id != "" -> {
      #(actions.load_single_category(id), IsLoading)
    }
    _, _ -> #(effect.none(), core.Idle)
  }
  let post_form = case new_route {
    PostEditPage("") -> Some(forms.make_post_form(None))
    _ -> model.post_form
  }
  let category_form = case new_route {
    CategoryEditPage("") -> Some(forms.make_category_form(None))
    _ -> model.category_form
  }
  let page_owned_objects = case new_route {
    PostEditPage(..) | CategoryEditPage(..) -> PageOwnedPosts([])
    _ -> model.page_owned_objects
  }
  let model =
    Model(
      ..model,
      route: new_route,
      login_state:,
      loading_status:,
      post_form:,
      category_form:,
      page_owned_objects:,
    )
  #(model, go_next)
}

pub fn handle_api_list_category_result(
  res: Result(ApiListingResponse(Category), rsvp.Error),
  model: Model,
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
          loading_status: core.Idle,
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
              loading_status: core.Idle,
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
  res: Result(Post, rsvp.Error),
  model: Model,
) {
  case res {
    Ok(p) -> {
      let form = forms.make_post_form(Some(p))
      let model =
        Model(
          ..model,
          post_form: Some(form),
          checkboxes: CheckBoxes(is_published: p.is_published),
          loading_status: core.Idle,
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
          loading_status: core.Idle,
        )
      #(model, effect.none())
    }
  }
}

pub fn handle_api_slug_generation(
  res: Result(String, rsvp.Error),
  model: Model,
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
  res: Result(PostEditablePart, Form(PostEditablePart)),
  stay: Bool,
  model: Model,
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
      io.println("Form errors:")
      echo formlib.all_errors(form)
      let post_form = model.post_form |> option.map(fn(_f) { form })
      #(Model(..model, post_form:), effect.none())
    }
  }
}

pub fn handle_api_update_post_result(
  res: Result(Post, rsvp.Error),
  stay: Bool,
  model: Model,
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
        False -> routes.goto(PostListPage(None, None, None))
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
  res: Result(Post, rsvp.Error),
  model: Model,
) {
  case res {
    Error(err) -> {
      let message = models.create_danger_message("Failed to save post.")
      let flash_messages = [message, ..model.flash_messages]
      let #(login_state, whatnext) = case err {
        rsvp.HttpError(Response(401, ..)) -> {
          io.println("Redirecting to Login page...")
          #(NonLogin, routes.goto(LoginPage))
        }
        _ -> #(model.login_state, effect.none())
      }
      #(Model(..model, flash_messages:, login_state:), whatnext)
    }
    Ok(post) -> {
      let message =
        models.create_success_message(
          "Post " <> post.title <> " has been created.",
        )
      let flash_messages = [message, ..model.flash_messages]
      let whatsnext =
        effect.batch([
          routes.goto(PostEditPage(post.id)),
          models.schedule_cleaning_flash_messages(),
        ])
      #(Model(..model, flash_messages:), whatsnext)
    }
  }
}

pub fn handle_category_moved_between_panes(
  id: String,
  to_move_in: Bool,
  model: Model,
) -> Model {
  let post_form =
    model.post_form
    |> option.map(push_in_or_out_category_from_form(_, id, to_move_in))
  Model(..model, post_form:)
}

fn push_in_or_out_category_from_form(
  form: Form(PostEditablePart),
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

pub fn handle_rendered_markdown_received(html: String, model: Model) {
  let model = Model(..model, post_body_preview: Some(html))
  let whatsnext = {
    use _dispatch, _root <- effect.after_paint
    ffi.show_dialog("." <> consts.selector_post_body_preview_dialog)
    Nil
  }
  #(model, whatsnext)
}

pub fn handle_submit_stay_button_clicked(
  button: Element,
  model: Model,
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

pub fn process_post_form_data_to_produce_msg(
  submitted_values: List(#(String, String)),
  form: Form(PostEditablePart),
  stay: Bool,
) {
  // If the checkbox is unchecked, the "is_published" field will not be in submitted data.
  // When the checkbox value is missing, we should clear its previous data from the "formal" form.
  // formal doesn't provide a function like remove_value, so we have to use set_values.

  // We are about to overwrite the data which `Form` is keeping.
  // Our rendered-HTML form is missing "categories" field, this field
  // will be absent in the submitted values.
  // So we need to retrieve it from the stored values.
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
  res: Result(Category, rsvp.Error),
  model: Model,
) {
  case res {
    Ok(cat) -> {
      let form = forms.make_category_form(Some(cat))
      let model =
        Model(..model, category_form: Some(form), loading_status: core.Idle)
      #(model, effect.none())
    }
    Error(_e) -> {
      let message = models.create_danger_message("Failed to load category")
      let Model(flash_messages:, ..) = model
      let model =
        Model(
          ..model,
          flash_messages: [message, ..flash_messages],
          loading_status: core.Idle,
        )
      #(model, effect.none())
    }
  }
}

pub fn handle_category_form_submission(
  res: Result(CategoryEditablePart, Form(CategoryEditablePart)),
  model: Model,
) {
  case res {
    Ok(data) -> {
      let whatsnext = case model.route {
        CategoryEditPage("") -> actions.create_category_via_api(data)
        CategoryEditPage(id) -> actions.update_category_via_api(id, data)
        _ -> effect.none()
      }
      #(model, whatsnext)
    }
    Error(form) -> {
      let category_form = model.category_form |> option.map(fn(_f) { form })
      #(Model(..model, category_form:), effect.none())
    }
  }
}

pub fn handle_api_update_category_result(
  res: Result(Category, rsvp.Error),
  model: Model,
) {
  case res {
    Ok(cat) -> {
      let message =
        models.create_success_message(
          "Category " <> cat.title <> " has been updated.",
        )
      let flash_messages = [message, ..model.flash_messages]
      #(
        Model(..model, flash_messages:),
        effect.batch([
          models.schedule_cleaning_flash_messages(),
          routes.goto(CategoryListPage(None)),
        ]),
      )
    }
    Error(_e) -> {
      let message = models.create_danger_message("Failed to save category.")
      let flash_messages = [message, ..model.flash_messages]
      #(
        Model(..model, flash_messages:),
        models.schedule_cleaning_flash_messages(),
      )
    }
  }
}

pub fn handle_api_create_category_result(
  res: Result(Category, rsvp.Error),
  model: Model,
) {
  case res {
    Ok(cat) -> {
      let message =
        models.create_success_message(
          "Category " <> cat.title <> " has been created.",
        )
      let flash_messages = [message, ..model.flash_messages]
      #(
        Model(..model, flash_messages:),
        effect.batch([
          models.schedule_cleaning_flash_messages(),
          routes.goto(CategoryEditPage(cat.id)),
        ]),
      )
    }
    Error(_e) -> {
      let message = models.create_danger_message("Failed to save category.")
      let flash_messages = [message, ..model.flash_messages]
      #(
        Model(..model, flash_messages:),
        models.schedule_cleaning_flash_messages(),
      )
    }
  }
}

pub fn handle_api_delete_content_item_result(
  res: Result(ContentItemId, rsvp.Error),
  model: Model,
) {
  case res {
    Ok(id) -> {
      let #(message, page_owned_objects) = case id, model.page_owned_objects {
        PostId(id), PageOwnedPosts(posts) -> {
          let message =
            posts
            |> list.find_map(fn(p) {
              case p.id == id {
                True -> Ok("Post " <> p.title <> " has been deleted.")
                _ -> Error(Nil)
              }
            })
            |> option.from_result
          // Remove from the list of objects
          let remaining =
            posts
            |> list.filter(fn(p) { p.id != id })
            |> PageOwnedPosts
          #(message, remaining)
        }
        core.CategoryId(id), PageOwnedCategories(cats) -> {
          let message =
            cats
            |> list.find_map(fn(c) {
              case c.id == id {
                True -> Ok("Category " <> c.title <> " has been deleted.")
                _ -> Error(Nil)
              }
            })
            |> option.from_result
          // Remove from the list of objects
          let remaining =
            cats
            |> list.filter(fn(c) { c.id != id })
            |> PageOwnedCategories
          #(message, remaining)
        }
        _, _ -> {
          #(None, model.page_owned_objects)
        }
      }
      let flash_messages = case
        message |> option.map(models.create_success_message)
      {
        Some(m) -> [m, ..model.flash_messages]
        None -> model.flash_messages
      }
      #(
        Model(..model, flash_messages:, page_owned_objects:),
        models.schedule_cleaning_flash_messages(),
      )
    }
    Error(_e) -> {
      let message = models.create_danger_message("Failed to delete.")
      let flash_messages = [message, ..model.flash_messages]
      #(
        Model(..model, flash_messages:),
        models.schedule_cleaning_flash_messages(),
      )
    }
  }
}
