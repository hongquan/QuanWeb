import gleam/bool
import gleam/http/response.{Response}
import gleam/io
import gleam/list
import gleam/option.{None, Some}
import gleam/order
import gleam/result
import gleam/string
import gleam/time/duration
import gleam/time/timestamp
import gleam/uri
import lustre
import lustre/effect.{type Effect}
import lustre/element.{type Element}
import lustre/element/html as h
import lustre/portal
import modem
import rsvp.{HttpError}
import store

import actions
import consts.{mounted_path}
import core.{
  ApiCreatedBook, ApiCreatedCategory, ApiCreatedPost, ApiCreatedPresentation,
  ApiDeletedContentItem, ApiLoginReturned, ApiRenderedMarkdown,
  ApiReturnedBookAuthors, ApiReturnedBooks, ApiReturnedCategories,
  ApiReturnedLogOutDone, ApiReturnedPosts, ApiReturnedPresentations,
  ApiReturnedSingleBook, ApiReturnedSingleCategory, ApiReturnedSinglePost,
  ApiReturnedSinglePresentation, ApiReturnedSlug, ApiReturnedUsers,
  ApiUpdatedBook, ApiUpdatedCategory, ApiUpdatedPost, ApiUpdatedPresentation,
  BookFormSubmitted, CategoryFormSubmitted, ContentItemDeletionClicked,
  FlashMessageTimeUp, FormCancelClicked, IsSubmitting, LogOutClicked, LoggedIn,
  NonLogin, OnRouteChange, PostFilterSubmitted, PostFormSubmitted,
  PresentationFormSubmitted, RouterInitDone, SlugGeneratorClicked,
  SubmitStayButtonClicked, TryingLogin, UserClickMarkdownPreview,
  UserConfirmedDeletion, UserMovedCategoryBetweenPane, UserSubmittedLoginForm,
}
import ffi
import forms.{create_login_form}
import models.{type AppMsg, type Model, Model, default_model}
import routes.{
  BookEditPage, BookListPage, CategoryEditPage, CategoryListPage, HomePage,
  LoginPage, PostEditPage, PostListPage, PresentationEditPage,
  PresentationListPage, on_url_change, parse_to_route,
}
import updates
import views/posts
import views/simple.{make_login_page}

pub fn main(base_path: String) -> Nil {
  let app = lustre.application(init, update, view)
  let assert Ok(_) = portal.register()
  let assert Ok(_a) = lustre.start(app, "#app", base_path)
  Nil
}

fn init(_args) -> #(Model, Effect(AppMsg)) {
  let #(path, query) =
    modem.initial_uri()
    |> result.map(fn(u) { #(u.path, u.query) })
    |> result.unwrap(#(mounted_path, None))
  let query =
    query
    |> option.map(uri.parse_query)
    |> option.to_result(Nil)
    |> result.flatten
    |> result.unwrap([])
  let route = parse_to_route(path, query)
  let #(saved_user, last_auth) = case store.load_user() |> option.from_result {
    Some(#(a, b)) -> #(Some(a), Some(b))
    _ -> #(None, None)
  }
  let auth_expired =
    {
      last_auth
      |> option.map(timestamp.add(_, duration.minutes(30)))
      |> option.map(timestamp.compare(_, timestamp.system_time()))
      |> option.unwrap(order.Lt)
    }
    == order.Lt
  let login_state = case route, saved_user, auth_expired {
    LoginPage(_u), _, _ -> TryingLogin(create_login_form())
    _, Some(user), False -> LoggedIn(user)
    _, _, _ -> NonLogin
  }
  let model = Model(..default_model, route:, login_state:)
  let route_react_setup = modem.init(on_url_change(_, OnRouteChange))
  let whatsnext =
    effect.batch([
      route_react_setup,
      {
        use dispatch, _root <- effect.before_paint
        dispatch(RouterInitDone)
      },
    ])
  #(model, whatsnext)
}

fn update(model: Model, msg: AppMsg) -> #(Model, Effect(AppMsg)) {
  let Model(route:, ..) = model
  case msg {
    RouterInitDone -> updates.handle_router_init_done(model)
    OnRouteChange(new_route) -> {
      case new_route {
        routes.External(url) -> {
          io.println("To go to external: " <> uri.to_string(url))
          #(model, modem.load(url))
        }
        _ -> updates.handle_landing_on_page(new_route, model)
      }
    }

    UserSubmittedLoginForm(form) -> {
      io.println("UserSubmittedLoginForm")
      updates.handle_login_submission(form, model)
    }
    ApiLoginReturned(res) -> updates.handle_login_api_result(res, model)
    ApiReturnedPosts(res) -> {
      let model = updates.handle_api_list_post_result(res, model)
      #(model, effect.none())
    }
    ApiReturnedCategories(res) ->
      updates.handle_api_list_category_result(res, model)
    ApiReturnedPresentations(res) -> {
      updates.handle_api_list_presentations_result(res, model)
    }
    ApiReturnedBooks(res) -> {
      updates.handle_api_list_books_result(res, model)
    }
    LogOutClicked -> {
      #(model, actions.initiate_logout())
    }
    ApiReturnedLogOutDone(Ok(_s)) -> {
      updates.handle_successful_logout(model)
    }
    PostFilterSubmitted(values) -> {
      let cleaned_data =
        values
        |> list.filter_map(fn(kv) {
          let #(k, v) = kv
          case v |> string.trim {
            "" -> Error(Nil)
            s -> Ok(#(k, s))
          }
        })
      let query = uri.query_to_string(cleaned_data)
      let #(path, _q) = routes.to_uri_parts(route) |> routes.prefix
      #(model, modem.push(path, Some(query), None))
    }
    ApiReturnedSinglePost(res) ->
      updates.handle_api_retrieve_post_result(res, model)
    SlugGeneratorClicked(title) -> #(
      model,
      actions.initiate_generate_slug(title),
    )
    ApiReturnedSlug(res) -> {
      #(updates.handle_api_slug_generation(res, model), effect.none())
    }

    PostFormSubmitted(result:, stay:) -> {
      updates.handle_post_form_submission(result, stay, model)
    }

    ApiCreatedPost(Error(HttpError(Response(401, ..))))
    | ApiUpdatedPost(Error(HttpError(Response(401, ..))), ..)
    | ApiCreatedCategory(Error(HttpError(Response(401, ..))))
    | ApiUpdatedCategory(Error(HttpError(Response(401, ..))))
    | ApiDeletedContentItem(Error(HttpError(Response(401, ..)))) -> {
      let attempt = routes.as_uri(model.route)
      let flash_messages = [
        models.create_info_message("Please login..."),
        ..model.flash_messages
      ]
      let model = Model(..model, login_state: NonLogin, flash_messages:)
      io.println("Redirecting to Login page...")
      echo LoginPage(attempt)
      #(model, routes.goto(LoginPage(attempt)))
    }
    ApiCreatedPost(res) -> updates.handle_api_create_post_result(res, model)
    ApiUpdatedPost(res, stay) ->
      updates.handle_api_update_post_result(res, stay, model)
    FlashMessageTimeUp -> {
      let flash_messages =
        model.flash_messages
        |> list.filter(fn(m) {
          m.created_at
          |> timestamp.add(duration.seconds(5))
          |> timestamp.compare(timestamp.system_time())
          == order.Gt
        })
      #(Model(..model, flash_messages:), effect.none())
    }
    UserMovedCategoryBetweenPane(id, selected) -> #(
      updates.handle_category_moved_between_panes(id, selected, model),
      effect.none(),
    )
    UserClickMarkdownPreview(s) -> {
      #(model, actions.try_render_markdown_via_api(s))
    }
    ApiRenderedMarkdown(Ok(html)) -> {
      updates.handle_rendered_markdown_received(html, model)
    }
    ApiReturnedUsers(Ok(users)) -> {
      let model = Model(..model, users:)
      #(model, effect.none())
    }

    SubmitStayButtonClicked(dom_element) -> {
      updates.handle_submit_stay_button_clicked(dom_element, model)
    }
    ApiReturnedSingleCategory(res) -> {
      updates.handle_api_retrieve_category_result(res, model)
    }
    FormCancelClicked -> {
      let whatsnext = case route {
        CategoryEditPage(..) -> routes.goto(CategoryListPage(None, None))
        PostEditPage(..) -> {
          routes.goto(PostListPage(None, None, None))
        }
        PresentationEditPage(..) -> {
          routes.goto(PresentationListPage(None))
        }
        BookEditPage(..) -> {
          routes.goto(BookListPage(None))
        }
        _ -> effect.none()
      }
      #(model, whatsnext)
    }
    CategoryFormSubmitted(res) -> {
      updates.handle_category_form_submission(res, model)
    }
    ApiUpdatedCategory(res) -> {
      updates.handle_api_update_category_result(res, model)
    }
    ApiCreatedCategory(res) -> {
      updates.handle_api_create_category_result(res, model)
    }
    ContentItemDeletionClicked(id) -> {
      // To show dialog for deletion confirmation
      let whatsnext = {
        use dispatch, _root <- effect.before_paint
        let agreed = ffi.confirm("Are you sure want to delete?")
        io.println("User agree? " <> bool.to_string(agreed))
        use <- bool.guard(!agreed, Nil)
        dispatch(UserConfirmedDeletion(id))
      }
      #(model, whatsnext)
    }
    UserConfirmedDeletion(id) -> {
      let model = Model(..model, loading_status: IsSubmitting)
      let whatnext = actions.delete_content_item_via_api(id)
      #(model, whatnext)
    }
    ApiDeletedContentItem(res) -> {
      updates.handle_api_delete_content_item_result(res, model)
    }
    // Presentation handlers
    ApiReturnedSinglePresentation(res) -> {
      updates.handle_api_retrieve_presentation_result(res, model)
    }
    PresentationFormSubmitted(res) -> {
      updates.handle_presentation_form_submission(res, model)
    }
    ApiCreatedPresentation(res) -> {
      updates.handle_api_create_presentation_result(res, model)
    }
    ApiUpdatedPresentation(res) -> {
      updates.handle_api_update_presentation_result(res, model)
    }
    // Book handlers
    ApiReturnedSingleBook(res) -> {
      updates.handle_api_retrieve_book_result(res, model)
    }
    BookFormSubmitted(res) -> {
      updates.handle_book_form_submission(res, model)
    }
    ApiCreatedBook(res) -> {
      updates.handle_api_create_book_result(res, model)
    }
    ApiUpdatedBook(res) -> {
      updates.handle_api_update_book_result(res, model)
    }
    ApiReturnedBookAuthors(res) -> {
      updates.handle_api_book_authors_result(res, model)
    }
    _ -> #(model, effect.none())
  }
}

fn view(model: Model) -> Element(AppMsg) {
  let Model(route:, login_state:, ..) = model
  case route, login_state {
    HomePage, _ -> {
      dummy_view()
    }
    LoginPage(_u), TryingLogin(form) ->
      make_login_page(model.loading_status, form, model.flash_messages)
    PostListPage(p, q, cat_id), _ -> {
      posts.render_post_table_page(option.unwrap(p, 1), q, cat_id, model)
    }
    PostEditPage(id), LoggedIn(_u) -> posts.render_post_edit_page(id, model)
    CategoryListPage(page, sort), _ -> {
      posts.render_category_table_page(option.unwrap(page, 1), sort, model)
    }
    CategoryEditPage(id), LoggedIn(_u) ->
      posts.render_category_edit_page(id, model)
    PresentationListPage(page), _ -> {
      posts.render_presentation_table_page(option.unwrap(page, 1), model)
    }
    PresentationEditPage(id), LoggedIn(_u) ->
      posts.render_presentation_edit_page(id, model)
    BookListPage(page), _ -> {
      posts.render_book_table_page(option.unwrap(page, 1), model)
    }
    BookEditPage(id), LoggedIn(_u) -> posts.render_book_edit_page(id, model)
    _, _ -> {
      echo route
      echo login_state
      dummy_view()
    }
  }
}

pub fn dummy_view() {
  h.div([], [h.h1([], [h.text("Hello")])])
}
