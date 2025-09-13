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

pub type LoginDetail {
  LoginDetail(email: String, password: String)
}

pub type Msg(r) {
  ApiReturnedPosts
  OnRouteChange(r)
}
