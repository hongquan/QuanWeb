import actions
import consts
import decoders
import ffi
import gleam/bool
import gleam/io
import gleam/json
import gleam/list
import gleam/option.{None, Some}
import gleam/order
import gleam/result
import gleam/string
import gleam/time/duration
import gleam/time/timestamp
import gleam/uri
import lustre
import lustre/effect.{type Effect}
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/portal
import modem
import plinth/javascript/storage

import core.{
  ApiCreatedCategory, ApiCreatedPost, ApiLoginReturned, ApiRenderedMarkdown,
  ApiReturnedCategories, ApiReturnedLogOutDone, ApiReturnedPosts,
  ApiReturnedSingleCategory, ApiReturnedSinglePost, ApiReturnedSlug,
  ApiReturnedUsers, ApiUpdatedCategory, ApiUpdatedPost, CategoryDeletionClicked,
  CategoryFormSubmitted, CheckBoxes, FlashMessageTimeUp, FormCancelClicked,
  IsSubmitting, LogOutClicked, LoggedIn, NonLogin, OnRouteChange,
  PostFilterSubmitted, PostFormSubmitted, RouterInitDone, SlugGeneratorClicked,
  SubmitStayButtonClicked, TryingLogin, UserClickMarkdownPreview,
  UserConfirmedCategoryDeletion, UserMovedCategoryBetweenPane,
  UserSubmittedLoginForm, UserToggledIsPublishedCheckbox,
}
import forms.{create_login_form}
import models.{type AppMsg, type Model, Model, default_model}
import routes.{
  CategoryEditPage, CategoryListPage, HomePage, LoginPage, PostEditPage,
  PostListPage, on_url_change, parse_to_route,
}
import updates
import views/posts
import views/simple.{make_login_page}

pub fn main(base_path: String) -> Nil {
  let app = lustre.application(init, update, view)
  let assert Ok(_) = portal.register()
  let assert Ok(_a) = lustre.start(app, "#app", base_path)
  Nil
}

fn init(mounted_path: String) -> #(Model, Effect(AppMsg)) {
  let #(path, query) =
    modem.initial_uri()
    |> result.map(fn(u) { #(u.path, u.query) })
    |> result.unwrap(#(mounted_path, None))
  let query =
    query
    |> option.map(uri.parse_query)
    |> option.to_result(Nil)
    |> result.flatten
    |> result.unwrap([])
  let route = parse_to_route(mounted_path, path, query)
  let saved_user =
    storage.local()
    |> result.map_error(fn(_e) {
      io.println_error("Failed to acquire localStorage!")
    })
    |> result.try(storage.get_item(_, consts.key_store_user))
    |> result.map_error(fn(_e) {
      io.println("user is not found in localStorage.")
    })
    |> result.try(fn(s) {
      json.parse(s, decoders.make_user_decoder())
      |> result.map_error(fn(e) {
        io.println_error("Failed to decode user.")
        echo e
        Nil
      })
    })
  let login_state = case route, saved_user {
    LoginPage, _ -> TryingLogin(create_login_form())
    _, Ok(user) -> LoggedIn(user)
    _, _ -> NonLogin
  }
  let model = Model(..default_model, mounted_path:, route:, login_state:)
  let route_react_setup =
    modem.init(on_url_change(_, mounted_path, OnRouteChange))
  let whatsnext =
    effect.batch([
      route_react_setup,
      {
        use dispatch, _root <- effect.before_paint
        dispatch(RouterInitDone)
      },
    ])
  #(model, whatsnext)
}

fn update(model: Model, msg: AppMsg) -> #(Model, Effect(AppMsg)) {
  io.println("In update()")
  let Model(route:, mounted_path:, ..) = model
  case msg {
    RouterInitDone -> updates.handle_router_init_done(model)
    OnRouteChange(new_route) -> updates.handle_landing_on_page(new_route, model)

    UserSubmittedLoginForm(form) -> {
      io.println("UserSubmittedLoginForm")
      updates.handle_login_submission(model, form)
    }
    ApiLoginReturned(res) -> updates.handle_login_api_result(model, res)
    ApiReturnedPosts(res) -> {
      let model = updates.handle_api_list_post_result(model, res)
      #(model, effect.none())
    }
    ApiReturnedCategories(res) ->
      updates.handle_api_list_category_result(model, res)
    LogOutClicked -> {
      #(model, actions.initiate_logout())
    }
    ApiReturnedLogOutDone(Ok(_s)) -> {
      updates.handle_successful_logout(model)
    }
    PostFilterSubmitted(values) -> {
      let cleaned_data =
        values
        |> list.filter_map(fn(kv) {
          let #(k, v) = kv
          case v |> string.trim {
            "" -> Error(Nil)
            s -> Ok(#(k, s))
          }
        })
      let query = uri.query_to_string(cleaned_data)
      let #(path, _q) =
        routes.to_uri_parts(route) |> routes.prefix(mounted_path)
      #(model, modem.push(path, Some(query), None))
    }
    ApiReturnedSinglePost(res) ->
      updates.handle_api_retrieve_post_result(model, res)
    SlugGeneratorClicked(title) -> #(
      model,
      actions.initiate_generate_slug(title),
    )
    ApiReturnedSlug(res) -> {
      #(updates.handle_api_slug_generation(model, res), effect.none())
    }
    PostFormSubmitted(result:, stay:) -> {
      updates.handle_post_form_submission(model, result, stay)
    }
    ApiCreatedPost(res) -> updates.handle_api_create_post_result(model, res)
    ApiUpdatedPost(res, stay) ->
      updates.handle_api_update_post_result(model, res, stay)
    FlashMessageTimeUp -> {
      let flash_messages =
        model.flash_messages
        |> list.filter(fn(m) {
          m.created_at
          |> timestamp.add(duration.seconds(5))
          |> timestamp.compare(timestamp.system_time())
          == order.Gt
        })
      #(Model(..model, flash_messages:), effect.none())
    }
    UserMovedCategoryBetweenPane(id, selected) -> #(
      updates.handle_category_moved_between_panes(model, id, selected),
      effect.none(),
    )
    UserClickMarkdownPreview(s) -> {
      #(model, actions.try_render_markdown_via_api(s))
    }
    ApiRenderedMarkdown(Ok(html)) -> {
      updates.handle_rendered_markdown_received(model, html)
    }
    ApiReturnedUsers(Ok(users)) -> {
      let model = Model(..model, users:)
      #(model, effect.none())
    }
    UserToggledIsPublishedCheckbox(checked) -> {
      let model = Model(..model, checkboxes: CheckBoxes(is_published: checked))
      #(model, effect.none())
    }
    SubmitStayButtonClicked(dom_element) -> {
      updates.handle_submit_stay_button_clicked(model, dom_element)
    }
    ApiReturnedSingleCategory(res) -> {
      updates.handle_api_retrieve_category_result(model, res)
    }
    FormCancelClicked -> {
      let whatsnext = case route {
        CategoryEditPage(..) ->
          routes.goto(CategoryListPage(None), mounted_path)
        PostEditPage(..) -> {
          routes.goto(PostListPage(None, None, None), mounted_path)
        }
        _ -> effect.none()
      }
      #(model, whatsnext)
    }
    CategoryFormSubmitted(res) -> {
      updates.handle_category_form_submission(model, res)
    }
    ApiUpdatedCategory(res) -> {
      updates.handle_api_update_category_result(model, res)
    }
    ApiCreatedCategory(res) -> {
      updates.handle_api_create_category_result(model, res)
    }
    CategoryDeletionClicked(id) if id != "" -> {
      io.println("CategoryDeletionClicked")
      // To show dialog for deletion confirmation
      let whatsnext = {
        use dispatch, _root <- effect.before_paint
        let agreed = ffi.confirm("Are you sure want to delete?")
        io.println("User agree? " <> bool.to_string(agreed))
        use <- bool.guard(!agreed, Nil)
        dispatch(UserConfirmedCategoryDeletion(id))
      }
      #(model, whatsnext)
    }
    UserConfirmedCategoryDeletion(id) if id != "" -> {
      let model = Model(..model, loading_status: IsSubmitting)
      #(model, actions.delete_category_via_api(id))
    }
    core.ApiDeletedCategory(res) -> {
      updates.handle_api_delete_category_result(res, model)
    }
    _ -> #(model, effect.none())
  }
}

fn view(model: Model) -> Element(AppMsg) {
  io.println("In view()")
  let Model(route:, login_state:, ..) = model
  case route, login_state {
    HomePage, _ -> {
      dummy_view()
    }
    LoginPage, TryingLogin(form) -> make_login_page(model.loading_status, form)
    PostListPage(p, q, cat_id), _ -> {
      posts.render_post_table_page(option.unwrap(p, 1), q, cat_id, model)
    }
    PostEditPage(id), LoggedIn(_u) -> posts.render_post_edit_page(id, model)
    CategoryListPage(page), _ -> {
      posts.render_category_table_page(option.unwrap(page, 1), model)
    }
    CategoryEditPage(id), LoggedIn(_u) ->
      posts.render_category_edit_page(id, model)
    _, _ -> {
      echo route
      echo login_state
      dummy_view()
    }
  }
}

pub fn dummy_view() {
  h.div([], [h.h1([], [h.text("Hello")])])
}
