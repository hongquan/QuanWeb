import gleam/bool
import gleam/dict
import gleam/int
import gleam/io
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/result
import gleam/string
import gleam/uri
import lustre/effect.{type Effect}
import modem

pub type Route {
  HomePage
  LoginPage
  PostListPage(page: Option(Int), q: Option(String), cat_id: Option(String))
  PostEditPage(id: String)
  CategoryListPage(page: Option(Int))
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
    "/logout", _ -> LoginPage
    "/posts/new", _ -> PostEditPage("")
    "/posts/" <> pid, _ -> PostEditPage(pid)
    "/posts", queries -> {
      let query_dict = dict.from_list(queries)
      let page =
        query_dict
        |> dict.get("page")
        |> result.try(int.parse)
        |> result.map(fn(p) {
          case p {
            n if n < 1 -> 1
            n -> n
          }
        })
        |> option.from_result
      let q = query_dict |> dict.get("q") |> option.from_result
      let cat_id = query_dict |> dict.get("cat_id") |> option.from_result
      PostListPage(page, q:, cat_id:)
    }
    _, _ -> {
      io.println("Unknown " <> path)
      NotFound
    }
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
    PostListPage(page, _q, _c) -> #(
      "/posts",
      page
        |> option.map(fn(p) { [#("page", int.to_string(p))] })
        |> option.map(uri.query_to_string),
    )
    PostEditPage("") -> #("/posts/new", None)
    PostEditPage(id) -> #("/posts/" <> id, None)
    _ -> #("/not-found", None)
  }
}

pub fn prefix(uri_parts: #(String, a), mounted_path: String) -> #(String, a) {
  let #(path, query) = uri_parts
  #({ mounted_path <> path } |> string.replace("//", "/"), query)
}

pub fn goto(route: Route, mounted_path: String) -> Effect(b) {
  let #(full_path, q) = to_uri_parts(route) |> prefix(mounted_path)
  modem.push(full_path, q, None)
}

pub fn as_url_string(route: Route, mounted_path: String) {
  let #(full_path, query) = to_uri_parts(route) |> prefix(mounted_path)
  case query {
    Some("") -> full_path
    Some(s) -> full_path <> "?" <> s
    _ -> full_path
  }
}

pub fn replace_page(
  query: List(#(String, String)),
  page: Int,
) -> List(#(String, String)) {
  let wo_page = query |> list.filter(fn(kv) { kv.0 != "page" })
  [#("page", int.to_string(page)), ..wo_page]
}
