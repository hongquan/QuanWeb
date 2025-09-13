import gleam/option.{type Option, None}
import gleam/result
import gleam/uri
import lustre
import lustre/effect.{type Effect}
import lustre/element.{type Element}
import lustre/element/html as h
import modem

import core.{type Msg, type User, OnRouteChange}
import routes.{
  type Route, HomePage, LoginPage, on_url_change, parse_to_route, to_uri_parts,
}
import views/simple.{make_login_page}

pub type Model {
  Model(route: Route, user: Option(User))
}

// `Msg` is generic with route type, we make concrete type here
type AppMsg =
  Msg(Route)

pub fn main() -> Nil {
  let app = lustre.application(init, update, view)
  let assert Ok(_a) = lustre.start(app, "#app", Nil)
  Nil
}

fn init(_args) -> #(Model, Effect(AppMsg)) {
  let #(path, query) =
    modem.initial_uri()
    |> result.map(fn(u) { #(u.path, u.query) })
    |> result.unwrap(#("/", None))
  let query =
    query
    |> option.map(uri.parse_query)
    |> option.to_result(Nil)
    |> result.flatten
    |> result.unwrap([])
  let route = parse_to_route(path, query)
  let model = Model(route, None)
  let whatsnext = [
    modem.init(on_url_change(_, OnRouteChange)),
    case option.is_none(model.user) {
      True -> {
        let #(p, q) = to_uri_parts(LoginPage)
        modem.push(p, q, None)
      }
      False -> effect.none()
    },
  ]
  #(model, effect.batch(whatsnext))
}

fn update(model: Model, msg: AppMsg) -> #(Model, Effect(AppMsg)) {
  echo model
  echo msg
  #(model, effect.none())
}

fn view(model: Model) -> Element(AppMsg) {
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
