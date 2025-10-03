import formal/form.{type Form}
import gleam/option.{type Option}
import gleam/time/timestamp.{type Timestamp}
import gleam/uri
import rsvp
import tempo.{type DateTime}

pub type Category {
  Category(id: String, title: String, title_vi: Option(String), slug: String)
}

// Post with a subset of fields, just enough
// to show on a list
pub type MiniPost {
  MiniPost(
    id: String,
    title: String,
    slug: String,
    is_published: Bool,
    created_at: DateTime,
    updated_at: DateTime,
    categories: List(Category),
  )
}

// Post with all fields
pub type Post {
  Post(
    id: String,
    title: String,
    slug: String,
    is_published: Bool,
    body: String,
    created_at: DateTime,
    updated_at: DateTime,
    categories: List(Category),
  )
}

pub type User {
  User(
    id: String,
    email: String,
    username: String,
    is_active: Bool,
    is_superuser: Bool,
  )
}

pub type LoginData {
  LoginData(email: String, password: String)
}

pub type LoginState {
  NonLogin
  TryingLogin(Form(LoginData))
  LoggedIn(User)
}

pub type ApiListingResponse(o) {
  ApiListingResponse(
    count: Int,
    objects: List(o),
    total_pages: Int,
    links: #(Option(uri.Uri), Option(uri.Uri)),
  )
}

// Objects to be rendered in a page
pub type PageOwnedObjects {
  PageOwnedPosts(List(MiniPost))
  PageOwnedCategories(List(Category))
}

pub type PageOwnedObjectPaging {
  PageOwnedObjectPaging(
    count: Int,
    total_pages: Int,
    links: #(Option(uri.Uri), Option(uri.Uri)),
  )
}

pub type Severity {
  Danger
  Warning
  Info
  Success
}

pub type FlashMessage {
  FlashMessage(
    content: String,
    severity: Severity,
    id: String,
    created_at: Timestamp,
  )
}

// The part of Post with editable fields
pub type PostEditablePart {
  PostEditablePart(
    title: String,
    slug: String,
    categories: List(String),
    body: String,
  )
}

pub type Msg(r) {
  RouterInitDone
  UserSubmittedLoginForm(Result(LoginData, Form(LoginData)))
  ApiLoginReturned(Result(User, rsvp.Error))
  ApiReturnedPosts(Result(ApiListingResponse(MiniPost), rsvp.Error))
  ApiReturnedCategories(Result(ApiListingResponse(Category), rsvp.Error))
  OnRouteChange(r)
  LogOutClicked
  ApiReturnedLogOutDone(Result(String, rsvp.Error))
  PostFilterSubmitted(List(#(String, String)))
  ApiReturnedSinglePost(Result(Post, rsvp.Error))
  PostFormSubmitted(Result(PostEditablePart, Form(PostEditablePart)))
  SlugGeneratorClicked(String)
  ApiReturnedSlug(Result(String, rsvp.Error))
  ApiUpdatedPost(Result(Post, rsvp.Error))
  ApiCreatedPost(Result(Post, rsvp.Error))
  FlashMessageTimeUp
  UserMovedCategoryBetweenPane(id: String, selected: Bool)
}
