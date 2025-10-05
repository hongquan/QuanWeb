import formal/form.{type Form}
import gleam/option.{type Option, None}
import gleam/time/timestamp
import lustre/effect
import plinth/javascript/global

import core.{
  type Category, type CheckBoxes, type FlashMessage, type LoginState,
  type MiniUser, type Msg, type PageOwnedObjectPaging, type PageOwnedObjects,
  type PostEditablePart, CheckBoxes, FlashMessage, FlashMessageTimeUp,
}
import routes.{type Route}
import utils

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
    users: List(MiniUser),
    post_form: Option(Form(PostEditablePart)),
    post_body_preview: Option(String),
    checkboxes: CheckBoxes,
  )
}

// `Msg` is generic with route type, we make concrete type here
pub type AppMsg =
  Msg(Route)

pub const default_checkboxes = CheckBoxes(is_published: False)

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
  users: [],
  post_form: None,
  post_body_preview: None,
  checkboxes: default_checkboxes,
)

pub fn create_success_message(content: String) {
  let id = utils.gen_simple_random_id()
  FlashMessage(
    content:,
    severity: core.Success,
    id:,
    created_at: timestamp.system_time(),
  )
}

pub fn create_info_message(content: String) {
  let id = utils.gen_simple_random_id()
  FlashMessage(
    content:,
    severity: core.Info,
    id:,
    created_at: timestamp.system_time(),
  )
}

pub fn create_danger_message(content: String) {
  let id = utils.gen_simple_random_id()
  FlashMessage(
    content:,
    severity: core.Danger,
    id:,
    created_at: timestamp.system_time(),
  )
}

pub fn schedule_cleaning_flash_messages() {
  use dispatch <- effect.from
  {
    use <- global.set_timeout(5000)
    dispatch(FlashMessageTimeUp)
  }
  Nil
}
