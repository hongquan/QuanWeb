import actions
import consts
import decoders
import gleam/io
import gleam/json
import gleam/option.{None, Some}
import gleam/result
import gleam/uri
import lustre
import lustre/effect.{type Effect}
import lustre/element.{type Element}
import lustre/element/html as h
import modem
import plinth/javascript/storage
import views/posts

import core.{
  ApiLoginReturned, ApiReturnedCategories, ApiReturnedLogOutDone,
  ApiReturnedPosts, LoggedIn, NonLogin, OnRouteChange, RouterInitDone,
  TryingLogin, UserSubmittedLoginForm,
}
import forms.{create_login_form}
import models.{type AppMsg, type Model, Model, default_model}
import routes.{HomePage, LoginPage, PostListPage, on_url_change, parse_to_route}
import updates
import views/simple.{make_login_page}

pub fn main(base_path: String) -> Nil {
  let app = lustre.application(init, update, view)
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
  let Model(
    route:,
    login_state:,
    mounted_path:,
    categories:,
    partial_load_categories:,
    ..,
  ) = model
  case msg {
    RouterInitDone -> {
      io.println("RouterInitDone")
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
        PostListPage(p, _q, _c), LoggedIn(_u) -> {
          let load_posts_action = actions.load_posts(option.unwrap(p, 1))
          let load_categories_action = case
            categories,
            partial_load_categories
          {
            [], _o -> actions.load_categories(1)
            _, _ -> effect.none()
          }
          #(effect.batch([load_posts_action, load_categories_action]), True)
        }
        // Already logged in, just serve, no redirect
        _, LoggedIn(_u) -> #(effect.none(), False)
        _, _ -> {
          #(routes.goto(LoginPage, mounted_path), False)
        }
      }
      let model = Model(..model, is_loading:)
      #(model, whatsnext)
    }
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
    core.LogOutClicked -> {
      #(model, actions.initiate_logout())
    }
    ApiReturnedLogOutDone(Ok(_s)) -> {
      let model = updates.handle_successful_logout(model)
      #(model, effect.none())
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
    LoginPage, TryingLogin(form) -> make_login_page(form)
    PostListPage(p, _q, _c), _ -> {
      posts.render_post_table_view(option.unwrap(p, 1), model)
    }
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
