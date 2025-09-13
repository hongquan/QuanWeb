import gleam/int
import gleam/list
import gleam/option.{type Option, None}
import gleam/result
import gleam/uri

pub type Route {
  HomePage
  LoginPage
  PostListPage(page: Int)
  PostEditPage(id: String)
  NotFound
}

pub fn parse_to_route(path: String, query: List(#(String, String))) -> Route {
  case path, query {
    "/", _ -> HomePage
    "/login", _ -> LoginPage
    "/posts", queries -> {
      let page =
        queries
        |> list.find_map(fn(x) {
          let #(name, value) = x
          case name {
            "page" -> int.parse(value)
            _ -> Error(Nil)
          }
        })
        |> result.map(fn(p) {
          case p {
            n if n < 1 -> 1
            n -> n
          }
        })
        |> result.unwrap(1)
      PostListPage(page)
    }
    _, _ -> NotFound
  }
}

pub fn on_url_change(url: uri.Uri, notify: fn(Route) -> msg) -> msg {
  let route =
    url.query
    |> option.map(fn(q) { option.from_result(uri.parse_query(q)) })
    |> option.flatten
    |> option.unwrap([])
    |> parse_to_route(url.path, _)
  notify(route)
}

pub fn to_uri_parts(route: Route) -> #(String, Option(String)) {
  case route {
    HomePage -> #("/", None)
    LoginPage -> #("/login", None)
    _ -> #("/not-found", None)
  }
}
