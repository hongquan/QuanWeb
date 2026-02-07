import gleam/bool
import gleam/dict
import gleam/int
import gleam/io
import gleam/list
import gleam/option.{type Option, None, Some}
import gleam/pair
import gleam/result
import gleam/string
import gleam/uri.{type Uri, Uri}
import lustre/effect.{type Effect}
import modem

pub type CategorySort {
  SortByTitle
  SortByFeatured
}

pub type Route {
  HomePage
  // The payload is the previous attempt URL
  LoginPage(Uri)
  PostListPage(page: Option(Int), q: Option(String), cat_id: Option(String))
  PostEditPage(id: String)
  CategoryListPage(page: Option(Int), sort: Option(CategorySort))
  CategoryEditPage(id: String)
  External(Uri)
  NotFound
}

import consts.{mounted_path}

pub fn parse_to_route(
  full_path: String,
  query: List(#(String, String)),
) -> Route {
  use <- bool.lazy_guard(!string.starts_with(full_path, mounted_path), fn() {
    uri.parse(full_path) |> result.map(External) |> result.unwrap(NotFound)
  })
  // If full_path == "/base/abc", we get "/abc"
  let path = string.drop_start(full_path, string.length(mounted_path) - 1)
  case path, query {
    "/", _ -> HomePage
    "/login", [] -> {
      LoginPage(uri.empty)
    }
    "/login", [#("attempt", s), ..] -> {
      LoginPage(uri.parse(s) |> result.unwrap(uri.empty))
    }
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
    "/categories", queries -> {
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
      let sort = case query_dict |> dict.get("sort") {
        Ok("featured") -> Some(SortByFeatured)
        _ -> None
      }
      CategoryListPage(page, sort)
    }
    "/categories/new", _ -> CategoryEditPage("")
    "/categories/" <> id, _ -> CategoryEditPage(id)
    _, _ -> {
      io.println("Unknown " <> path)
      NotFound
    }
  }
}

pub fn on_url_change(url: Uri, notify: fn(Route) -> msg) -> msg {
  let route =
    url.query
    |> option.map(fn(q) { option.from_result(uri.parse_query(q)) })
    |> option.flatten
    |> option.unwrap([])
    |> parse_to_route(url.path, _)
  io.println("URL changed to: " <> uri.to_string(url))
  notify(route)
}

/// Return path and query string for a Route instance.
/// The mounted path is not taken into account.
pub fn to_uri_parts(route: Route) -> #(String, Option(String)) {
  case route {
    HomePage -> #("/", None)
    LoginPage(attempt) -> {
      let query = case uri.to_string(attempt) {
        "" -> None
        u -> uri.query_to_string([#("attempt", u)]) |> Some
      }
      #("/login", query)
    }
    PostListPage(page, q, cat_id) -> {
      let query =
        [
          cat_id |> option.map(pair.new("cat_id", _)),
          q |> option.map(pair.new("q", _)),
          page |> option.map(int.to_string) |> option.map(pair.new("page", _)),
        ]
        |> option.values
        |> uri.query_to_string
      #("/posts", Some(query))
    }
    PostEditPage("") -> #("/posts/new", None)
    PostEditPage(id) -> #("/posts/" <> id, None)
    CategoryListPage(page, sort) -> {
      let query_params = case page {
        Some(p) -> [#("page", int.to_string(p))]
        None -> []
      }
      let query_params = case sort {
        Some(SortByFeatured) -> [#("sort", "featured"), ..query_params]
        _ -> query_params
      }
      let query = case query_params {
        [] -> None
        _ -> uri.query_to_string(query_params) |> Some
      }
      #("/categories", query)
    }
    CategoryEditPage("") -> #("/categories/new", None)
    CategoryEditPage(id) -> #("/categories/" <> id, None)
    _ -> #("/not-found", None)
  }
}

fn to_page_query(page: Option(Int)) {
  page |> option.map(int.to_string) |> option.map(pair.new("page", _))
  case page {
    Some(p) ->
      int.to_string(p)
      |> pair.new("page", _)
      |> list.wrap
      |> uri.query_to_string
      |> Some
    None -> None
  }
  page
  |> option.map(fn(p) { [#("page", int.to_string(p))] })
  |> option.map(uri.query_to_string)
}

pub fn prefix(uri_parts: #(String, a)) -> #(String, a) {
  let #(path, query) = uri_parts
  #({ mounted_path <> path } |> string.replace("//", "/"), query)
}

pub fn goto(route: Route) -> Effect(b) {
  let #(full_path, q) = to_uri_parts(route) |> prefix
  modem.push(full_path, q, None)
}

pub fn as_url_string(route: Route) {
  let #(full_path, query) = to_uri_parts(route) |> prefix
  case query {
    Some("") -> full_path
    Some(s) -> full_path <> "?" <> s
    _ -> full_path
  }
}

pub fn as_uri(route: Route) -> Uri {
  let #(full_path, query) = to_uri_parts(route) |> prefix
  Uri(..uri.empty, path: full_path, query:)
}

pub fn replace_page(
  query: List(#(String, String)),
  page: Int,
) -> List(#(String, String)) {
  let wo_page = query |> list.filter(fn(kv) { kv.0 != "page" })
  [#("page", int.to_string(page)), ..wo_page]
}

pub fn are_routes_matched(current: Route, link: Route) {
  case current, link {
    HomePage, HomePage -> True
    PostListPage(..), PostListPage(..) -> True
    PostEditPage(..), PostListPage(..) -> True
    CategoryListPage(..), CategoryListPage(..) -> True
    CategoryEditPage(..), CategoryListPage(..) -> True
    _, _ -> False
  }
}
