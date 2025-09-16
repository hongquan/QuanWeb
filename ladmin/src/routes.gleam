import gleam/bool
import gleam/int
import gleam/io
import gleam/list
import gleam/option.{type Option, None}
import gleam/result
import gleam/string
import gleam/uri

pub type Route {
  HomePage
  LoginPage
  PostListPage(page: Int)
  PostEditPage(id: String)
  NotFound
}

pub fn parse_to_route(
  mounted_path: String,
  full_path: String,
  query: List(#(String, String)),
) -> Route {
  use <- bool.guard(!string.starts_with(full_path, mounted_path), NotFound)
  // If full_path == "/base/abc", we get "/abc"
  let path = string.drop_start(full_path, string.length(mounted_path) - 1)
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

pub fn on_url_change(
  url: uri.Uri,
  mounted_path: String,
  notify: fn(Route) -> msg,
) -> msg {
  let route =
    url.query
    |> option.map(fn(q) { option.from_result(uri.parse_query(q)) })
    |> option.flatten
    |> option.unwrap([])
    |> parse_to_route(mounted_path, url.path, _)
  io.println("URL changed to: " <> uri.to_string(url))
  notify(route)
}

/// Return path and query string for a Route instance.
/// The mounted path is not taken into account.
pub fn to_uri_parts(route: Route) -> #(String, Option(String)) {
  case route {
    HomePage -> #("/", None)
    LoginPage -> #("/login", None)
    _ -> #("/not-found", None)
  }
}

pub fn prefix(path: String, mounted_path: String) {
  { mounted_path <> path } |> string.replace("//", "/")
}
