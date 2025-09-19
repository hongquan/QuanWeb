import actions
import consts
import decoders
import gleam/io
import gleam/json
import gleam/option.{None}
import gleam/result
import gleam/uri
import lustre
import lustre/effect.{type Effect}
import lustre/element.{type Element}
import lustre/element/html as h
import modem
import plinth/javascript/storage

import core.{
  ApiLoginReturned, LoggedIn, NonLogin, OnRouteChange, RouterInitDone,
  TryingLogin, UserSubmittedLoginForm,
}
import forms.{create_login_form}
import models.{type AppMsg, type Model, Model}
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
  let model = Model(mounted_path, route, login_state:)
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
  let Model(route:, login_state:, mounted_path:) = model
  case msg {
    RouterInitDone -> {
      io.println("RouterInitDone")
      echo route
      let whatsnext = case route, login_state {
        LoginPage, _ -> effect.none()
        // If user has already logged-in, and visiting HomePage, redirect to PostList
        HomePage, LoggedIn(_u) -> {
          routes.goto(PostListPage(1), mounted_path)
        }
        // Already logged in, just serve, no redirect
        _, LoggedIn(_u) -> effect.none()
        _, _ -> {
          routes.goto(LoginPage, mounted_path)
        }
      }
      #(model, whatsnext)
    }
    OnRouteChange(new_route) -> {
      let login_state = case new_route, model.login_state {
        LoginPage, NonLogin -> TryingLogin(create_login_form())
        _, state -> state
      }
      let go_next = case new_route, login_state {
        // If user has logged-in, redirect to "/posts" page
        HomePage, LoggedIn(_u) -> {
          routes.goto(PostListPage(1), mounted_path)
        }
        PostListPage(p), _ -> {
          actions.load_posts(p)
        }
        _, _ -> effect.none()
      }
      let model = Model(..model, route: new_route, login_state:)
      #(model, go_next)
    }
    UserSubmittedLoginForm(form) -> {
      io.println("UserSubmittedLoginForm")
      updates.handle_login_submission(model, form)
    }
    ApiLoginReturned(res) -> updates.handle_login_api_result(model, res)
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
