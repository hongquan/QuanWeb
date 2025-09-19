import formal/form.{type Form}
import gleam/option.{type Option}
import gleam/uri
import rsvp
import tempo.{type DateTime}

pub type Post {
  Post(
    id: String,
    title: String,
    slug: String,
    is_published: Bool,
    created_at: DateTime,
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

pub type Category {
  Category(id: String, title: String)
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
  PageOwnedPosts(List(Post))
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
  FlashMessage(content: String, severity: Severity)
}

pub type Msg(r) {
  RouterInitDone
  UserSubmittedLoginForm(Result(LoginData, Form(LoginData)))
  ApiLoginReturned(Result(User, rsvp.Error))
  ApiReturnedPosts(Result(ApiListingResponse(Post), rsvp.Error))
  OnRouteChange(r)
}
