import gleam/io
import gleam/option.{None}
import gleam/result
import gleam/string
import gleam/uri
import lustre
import lustre/effect.{type Effect}
import lustre/element.{type Element}
import lustre/element/html as h
import modem

import core.{type LoginState, type Msg, NonLogin, OnRouteChange, RouterInitDone}
import routes.{
  type Route, HomePage, LoginPage, on_url_change, parse_to_route, to_uri_parts,
}
import views/simple.{make_login_page}

pub type Model {
  Model(mounted_path: String, route: Route, login_state: LoginState)
}

// `Msg` is generic with route type, we make concrete type here
type AppMsg =
  Msg(Route)

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
  let model = Model(mounted_path, route, login_state: NonLogin)
  let route_react_setup =
    modem.init(on_url_change(_, mounted_path, OnRouteChange))
  case model.login_state {
    NonLogin -> #(
      model,
      effect.batch([
        route_react_setup,
        {
          use dispatch, _root <- effect.before_paint
          dispatch(RouterInitDone)
        },
      ]),
    )
    _ -> #(model, route_react_setup)
  }
}

fn update(model: Model, msg: AppMsg) -> #(Model, Effect(AppMsg)) {
  io.println("In update()")
  case msg {
    RouterInitDone -> {
      let whatsnext = case model.route {
        LoginPage -> effect.none()
        _ -> {
          let #(p, q) = to_uri_parts(LoginPage)
          let full_path = string.replace(model.mounted_path <> p, "//", "/")
          modem.push(full_path, q, None)
        }
      }
      #(model, whatsnext)
    }
    OnRouteChange(new_route) -> {
      let model = Model(..model, route: new_route)
      #(model, effect.none())
    }
    _ -> #(model, effect.none())
  }
}

fn view(model: Model) -> Element(AppMsg) {
  io.println("In view()")
  let route = model.route
  case route {
    HomePage -> dummy_view()
    LoginPage -> make_login_page()
    _ -> dummy_view()
  }
}

pub fn dummy_view() {
  h.div([], [h.h1([], [h.text("Hello")])])
}
