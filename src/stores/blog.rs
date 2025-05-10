use std::collections::HashMap;

use gel_protocol::model::Datetime as EDatetime;
use gel_protocol::named_args;
use gel_protocol::value_opt::ValueOpt;
use gel_tokio::{Client, Error};
use smallvec::SmallVec;
use str_macro::str;
use tracing::{debug, info};
use uuid::Uuid;

use crate::models::{BlogCategory, DetailedBlogPost, MediumBlogPost, MiniBlogPost};
use crate::types::EdgeSelectable;

pub async fn count_search_result_posts(
    lower_search_tokens: Option<&Vec<String>>,
    client: &Client,
) -> Result<usize, Error> {
    let lower_search_tokens: Option<Vec<&str>> =
        lower_search_tokens.map(|v| v.iter().map(|s| s.as_str()).collect());
    let filter_line = if lower_search_tokens.is_some() {
        "FILTER all(contains(str_lower(BlogPost.title), array_unpack(<array<str>>$0)))"
    } else {
        ""
    };
    let q = format!("SELECT count((SELECT BlogPost {filter_line}))");
    tracing::debug!("To query: {}", q);
    let count: i64 = if let Some(tokens) = lower_search_tokens {
        tracing::debug!("With args: {:?}", tokens);
        client.query_required_single(&q, &(tokens,)).await?
    } else {
        client.query_required_single(&q, &()).await?
    };
    Ok(count.try_into().unwrap_or(0))
}

pub async fn count_all_published_posts(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count((SELECT BlogPost FILTER .is_published = true))";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_post(post_id: Uuid, client: &Client) -> Result<Option<DetailedBlogPost>, Error> {
    // Note: For now, we cannot use Gel splats syntax because the returned field order
    // does not match DetailedBlogPost.
    let fields = DetailedBlogPost::fields_as_shape();
    let q = format!(
        "SELECT BlogPost {fields}
        FILTER .id = <uuid>$0"
    );
    tracing::debug!("To query: {}", q);
    let post: Option<DetailedBlogPost> = client.query_single(&q, &(post_id,)).await?;
    Ok(post)
}

pub async fn get_detailed_post_by_slug(
    slug: String,
    client: &Client,
) -> Result<Option<DetailedBlogPost>, Error> {
    // Note: For now, we cannot use Gel splats syntax because the returned field order
    // does not match DetailedBlogPost.
    let fields = DetailedBlogPost::fields_as_shape();
    let q = format!(
        "SELECT BlogPost {fields}
        FILTER .slug = <str>$0"
    );
    tracing::debug!("To query: {}", q);
    let post: Option<DetailedBlogPost> = client.query_single(&q, &(slug,)).await?;
    Ok(post)
}

pub async fn get_blogposts(
    lower_search_tokens: Option<&Vec<String>>,
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<MediumBlogPost>, Error> {
    let filter_line = if lower_search_tokens.is_some() {
        "FILTER all(contains(str_lower(.title), array_unpack(<array<str>>$tokens)))"
    } else {
        ""
    };
    let mut args = HashMap::new();
    let mut paging_lines: SmallVec<[_; 2]> = SmallVec::new();
    if let Some(ss) = lower_search_tokens {
        let v: Vec<&str> = ss.iter().map(|s| s.as_str()).collect();
        args.insert("tokens", ValueOpt::from(v));
    }
    if let Some(offset) = offset {
        args.insert("offset", ValueOpt::from(offset));
        paging_lines.push(str!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        args.insert("limit", ValueOpt::from(limit));
        paging_lines.push(str!("LIMIT <int64>$limit"));
    }
    let paging_expr = paging_lines.join(" ");
    let fields = MediumBlogPost::fields_as_shape();
    let q = format!(
        "SELECT BlogPost {fields}
        {filter_line}
        ORDER BY .created_at DESC EMPTY FIRST {paging_expr}"
    );
    debug!("To query: {q}");
    debug!("With args: {args:?}");
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn get_published_posts(
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<MediumBlogPost>, Error> {
    let mut args = HashMap::with_capacity(2);
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    if let Some(offset) = offset {
        args.insert("offset", ValueOpt::from(offset));
        paging_lines.push(str!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        args.insert("limit", ValueOpt::from(limit));
        paging_lines.push(str!("LIMIT <int64>$limit"));
    }
    let paging_expr = paging_lines.join(" ");
    let fields = MediumBlogPost::fields_as_shape();
    let q = format!(
        "SELECT BlogPost {fields}
        FILTER .is_published = true ORDER BY .created_at DESC EMPTY FIRST {paging_expr}"
    );
    info!("To query: {q}");
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn get_published_posts_under_category(
    cat_slug: Option<String>,
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<MediumBlogPost>, Error> {
    let mut filter_lines = vec![".is_published = true"];
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    let mut args: HashMap<&str, ValueOpt> = HashMap::new();
    if let Some(slug) = cat_slug {
        filter_lines.push(".categories.slug = <str>$slug");
        args.insert("slug", slug.into());
    }
    if let Some(offset) = offset {
        args.insert("offset", offset.into());
        paging_lines.push(str!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        args.insert("limit", limit.into());
        paging_lines.push(str!("LIMIT <int64>$limit"));
    }
    let filter_expr = filter_lines.join(" AND ");
    let paging_expr = paging_lines.join(" ");
    let fields = MediumBlogPost::fields_as_shape();

    let q = format!(
        "SELECT BlogPost {fields}
        FILTER {filter_expr} ORDER BY .created_at DESC EMPTY FIRST {paging_expr}"
    );
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn count_blogposts_under_category(id: Uuid, client: &Client) -> Result<usize, Error> {
    let q = "
    SELECT count((SELECT BlogPost FILTER .categories.id = <uuid>$0))";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &(id,)).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_published_uncategorized_blogposts(
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<MediumBlogPost>, Error> {
    let mut args: HashMap<&str, ValueOpt> = HashMap::with_capacity(2);
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    if let Some(offset) = offset {
        args.insert("offset", offset.into());
        paging_lines.push(str!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        args.insert("limit", limit.into());
        paging_lines.push(str!("LIMIT <int64>$limit"));
    }
    let paging_expr = paging_lines.join(" ");
    let fields = MediumBlogPost::fields_as_shape();
    let q = format!("
    SELECT BlogPost {fields}
    FILTER .is_published = true AND NOT EXISTS .categories ORDER BY .created_at DESC EMPTY FIRST {paging_expr}");
    debug!("To query: {q}");
    debug!("With args: {args:#?}");
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn count_published_uncategorized_posts(client: &Client) -> Result<usize, Error> {
    let q = "
    SELECT count((SELECT BlogPost FILTER .is_published = true AND NOT EXISTS .categories))";
    debug!("To query: {q}");
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_blog_categories(
    offset: Option<i64>,
    limit: Option<i64>,
    client: &Client,
) -> Result<Vec<BlogCategory>, Error> {
    let q = format!(
        "SELECT BlogCategory {} ORDER BY .title OFFSET <optional int64>$0 LIMIT <optional int64>$1",
        BlogCategory::fields_as_shape()
    );
    let categories: Vec<BlogCategory> = client.query(&q, &(offset, limit)).await?;
    Ok(categories)
}

pub async fn get_all_categories_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(BlogCategory)";
    debug!("To query: {q}");
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_category(id: Uuid, client: &Client) -> Result<Option<BlogCategory>, Error> {
    let q = format!(
        "SELECT BlogCategory {} FILTER .id = <uuid>$0",
        BlogCategory::fields_as_shape()
    );
    tracing::debug!("To query: {}", q);
    let cat: Option<BlogCategory> = client.query_single(&q, &(id,)).await?;
    Ok(cat)
}

pub async fn get_category_by_slug(
    slug: &str,
    client: &Client,
) -> Result<Option<BlogCategory>, Error> {
    let q = format!(
        "SELECT BlogCategory {} FILTER .slug = <str>$0",
        BlogCategory::fields_as_shape()
    );
    tracing::debug!("To query: {}", q);
    let cat: Option<BlogCategory> = client.query_single(&q, &(slug,)).await?;
    Ok(cat)
}

pub async fn get_previous_post(
    created_at: EDatetime,
    cat_slug: Option<&str>,
    client: &Client,
) -> Result<Option<MiniBlogPost>, Error> {
    let mut filter_lines = vec![
        ".created_at < <datetime>$created_at",
        ".is_published = true",
    ];
    let mut args = named_args! {
        "created_at" => created_at
    };
    if let Some(slug) = cat_slug {
        filter_lines.push(".categories.slug = <str>$slug");
        args.insert("slug", slug.into());
    }
    let filter_expr = filter_lines.join(" AND ");
    let fields = MiniBlogPost::fields_as_shape();

    let q =
        format!("SELECT BlogPost {fields} FILTER {filter_expr} ORDER BY .created_at DESC LIMIT 1");
    debug!("To query: {q}");
    let post: Option<MiniBlogPost> = client.query_single(&q, &args).await?;
    Ok(post)
}

pub async fn get_next_post(
    created_at: EDatetime,
    cat_slug: Option<&str>,
    client: &Client,
) -> Result<Option<MiniBlogPost>, Error> {
    let mut filter_lines = vec![
        ".created_at > <datetime>$created_at",
        ".is_published = true",
    ];
    let mut args = named_args! {
        "created_at" => created_at
    };
    if let Some(slug) = cat_slug {
        filter_lines.push(".categories.slug = <str>$slug");
        args.insert("slug", slug.into());
    }
    let filter_expr = filter_lines.join(" AND ");

    let fields = MiniBlogPost::fields_as_shape();
    let q =
        format!("SELECT BlogPost {fields} FILTER {filter_expr} ORDER BY .created_at ASC LIMIT 1");
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(&q, &args).await?;
    Ok(post)
}

pub async fn get_last_updated_post(client: &Client) -> Result<Option<MiniBlogPost>, Error> {
    let q = format!(
        "SELECT BlogPost {} FILTER .is_published = true ORDER BY .updated_at DESC LIMIT 1",
        MiniBlogPost::fields_as_shape()
    );
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(&q, &()).await?;
    Ok(post)
}

pub async fn get_mini_post_by_old_id(
    old_id: u32,
    client: &Client,
) -> Result<Option<MiniBlogPost>, Error> {
    let field_names = MiniBlogPost::fields_as_shape();
    let q = format!("SELECT BlogPost {field_names} FILTER .old_id = <int32>$0");
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(&q, &(old_id as i32,)).await?;
    Ok(post)
}
