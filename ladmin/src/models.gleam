import gleam/option.{type Option, None}

import core.{
  type Category, type FlashMessage, type LoginState, type Msg,
  type PageOwnedObjectPaging, type PageOwnedObjects, type Post,
}
import routes.{type Route}

pub type Model {
  Model(
    mounted_path: String,
    route: Route,
    login_state: LoginState,
    page_owned_objects: PageOwnedObjects,
    page_owned_object_paging: PageOwnedObjectPaging,
    flash_messages: List(FlashMessage),
    is_loading: Bool,
    categories: List(Category),
    partial_load_categories: List(Category),
    editing_post: Option(Post),
  )
}

// `Msg` is generic with route type, we make concrete type here
pub type AppMsg =
  Msg(Route)

pub const default_model = Model(
  mounted_path: "/",
  route: routes.HomePage,
  login_state: core.NonLogin,
  page_owned_objects: core.PageOwnedPosts([]),
  page_owned_object_paging: core.PageOwnedObjectPaging(
    count: 0,
    total_pages: 0,
    links: #(None, None),
  ),
  flash_messages: [],
  is_loading: False,
  categories: [],
  partial_load_categories: [],
  editing_post: None,
)

pub fn create_success_message(content: String) {
  core.FlashMessage(content:, severity: core.Success)
}

pub fn create_info_message(content: String) {
  core.FlashMessage(content:, severity: core.Info)
}

pub fn create_danger_message(content: String) {
  core.FlashMessage(content:, severity: core.Danger)
}
