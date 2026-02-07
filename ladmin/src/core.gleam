import formal/form.{type Form}
import gleam/option.{type Option}
import gleam/time/timestamp.{type Timestamp}
import gleam/uri
import plinth/browser/element.{type Element}
import rsvp

pub type Category {
  Category(
    id: String,
    title: String,
    title_vi: Option(String),
    slug: String,
    header_color: Option(String),
    summary_en: Option(String),
    summary_vi: Option(String),
  )
}

// Post with a subset of fields, just enough
// to show on a list
pub type MiniPost {
  MiniPost(
    id: String,
    title: String,
    slug: String,
    is_published: Bool,
    created_at: Timestamp,
    updated_at: Timestamp,
    categories: List(Category),
  )
}

// Part of User with just few fields to render dropdown
pub type MiniUser {
  MiniUser(id: String, email: String)
}

// Post with all fields
pub type Post {
  Post(
    id: String,
    title: String,
    slug: String,
    is_published: Bool,
    body: Option(String),
    created_at: Timestamp,
    updated_at: Timestamp,
    categories: List(Category),
    locale: Option(String),
    author: Option(MiniUser),
    og_image: Option(String),
    seo_description: Option(String),
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

pub type LoadingStatus {
  Idle
  IsLoading
  IsSubmitting
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
  FlashMessage(content: String, severity: Severity, created_at: Timestamp)
}

// The part of Post with editable fields
pub type PostEditablePart {
  PostEditablePart(
    title: String,
    slug: String,
    categories: List(String),
    body: Option(String),
    locale: Option(String),
    author: String,
    is_published: Bool,
    og_image: Option(String),
  )
}

pub type CategoryEditablePart {
  CategoryEditablePart(
    title: String,
    slug: String,
    title_vi: Option(String),
    header_color: Option(String),
    summary_en: Option(String),
    summary_vi: Option(String),
  )
}

pub type Color {
  Blue
  Sky
  Purple
}

pub type ContentItemId {
  PostId(String)
  CategoryId(String)
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
  PostFormSubmitted(
    result: Result(PostEditablePart, Form(PostEditablePart)),
    stay: Bool,
  )
  SlugGeneratorClicked(String)
  ApiReturnedSlug(Result(String, rsvp.Error))
  ApiUpdatedPost(result: Result(Post, rsvp.Error), stay: Bool)
  ApiCreatedPost(Result(Post, rsvp.Error))
  FlashMessageTimeUp
  UserMovedCategoryBetweenPane(id: String, selected: Bool)
  UserClickMarkdownPreview(text: String)
  ApiRenderedMarkdown(Result(String, rsvp.Error))
  ApiReturnedUsers(Result(List(MiniUser), rsvp.Error))
  SubmitStayButtonClicked(Element)
  ApiReturnedSingleCategory(Result(Category, rsvp.Error))
  FormCancelClicked
  CategoryFormSubmitted(
    result: Result(CategoryEditablePart, Form(CategoryEditablePart)),
  )
  ApiCreatedCategory(Result(Category, rsvp.Error))
  ApiUpdatedCategory(Result(Category, rsvp.Error))
  ContentItemDeletionClicked(ContentItemId)
  UserConfirmedDeletion(ContentItemId)
  ApiDeletedContentItem(Result(ContentItemId, rsvp.Error))
}
