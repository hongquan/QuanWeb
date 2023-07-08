use uuid::Uuid;
use edgedb_tokio::{Client, Error};
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_protocol::value::Value as EValue;
use edgedb_protocol::common::Cardinality as Cd;
use indexmap::indexmap;

use crate::models::blogs::MiniBlogPost;
use crate::models::{RawBlogPost, DetailedBlogPost, BlogCategory};
use crate::types::conversions::{edge_object_from_simple_pairs, edge_object_from_pairs};

pub async fn get_all_posts_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(BlogPost)";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_blogpost(post_id: Uuid, client: &Client) -> Result<Option<DetailedBlogPost>, Error> {
    // Note: For now, we cannot use EdgeDB splats syntax because the returned field order
    // does not match DetailedBlogPost.
    let q = "
    SELECT BlogPost {
        id,
        title,
        slug,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {id, title, slug},
        body,
        format,
        locale,
        excerpt,
        html,
        seo_description,
        og_image,
    }
    FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<DetailedBlogPost> = client.query_single(q, &(post_id,)).await?;
    Ok(post)
}

pub async fn get_blogpost_by_slug(slug: String, client: &Client) -> Result<Option<DetailedBlogPost>, Error> {
    // Note: For now, we cannot use EdgeDB splats syntax because the returned field order
    // does not match DetailedBlogPost.
    let q = "
    SELECT BlogPost {
        id,
        title,
        slug,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {id, title, slug},
        body,
        format,
        locale,
        excerpt,
        html,
        seo_description,
        og_image,
    }
    FILTER .slug = <str>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<DetailedBlogPost> = client.query_single(q, &(slug,)).await?;
    Ok(post)
}

pub async fn get_blogposts(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<RawBlogPost>, Error> {
    let q = "
    SELECT BlogPost {
        id,
        title,
        slug,
        excerpt,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {
            id,
            title,
            slug,
        },
    }
    ORDER BY .created_at DESC EMPTY FIRST OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    let posts: Vec<RawBlogPost> = client.query(q, &(offset, limit)).await?;
    Ok(posts)
}

pub async fn get_blogposts_under_category(cat_slug: Option<String>, offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<RawBlogPost>, Error> {
    let mut filter_lines = vec![
        ".is_published = true",
    ];
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    let mut pairs = indexmap! {};
    if let Some(slug) = cat_slug {
        filter_lines.push(".categories.slug = <str>$slug");
        pairs.insert("slug", (Some(EValue::Str(slug)), Cd::One));
    }
    if let Some(offset) = offset {
        pairs.insert("offset", (Some(EValue::Int64(offset)), Cd::One));
        paging_lines.push(format!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        pairs.insert("limit", (Some(EValue::Int64(limit)), Cd::One));
        paging_lines.push(format!("LIMIT <int64>$limit"));
    }
    let filter_expr = filter_lines.join(" AND ");
    let paging_expr = paging_lines.join(" ");
    let args = edge_object_from_pairs(pairs);

    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        excerpt,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {{
            id,
            title,
            slug,
        }},
    }}
    FILTER {filter_expr} ORDER BY .created_at DESC EMPTY FIRST {paging_expr}");
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let posts: Vec<RawBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn count_blogposts_under_category(id: Uuid, client: &Client) -> Result<usize, Error> {
    let q = "
    SELECT count((SELECT BlogPost FILTER .categories.id = <uuid>$0))";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &(id,)).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_blog_categories(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<BlogCategory>, Error> {
    let q = "
    SELECT BlogCategory {
        id,
        title,
        slug
    } ORDER BY .title OFFSET <optional int64>$0 LIMIT <optional int64>$1";
    let categories: Vec<BlogCategory> = client.query(q, &(offset, limit)).await?;
    Ok(categories)
}


pub async fn get_all_categories_count(client: &Client) -> Result<usize, Error> {
    let q = "SELECT count(BlogCategory)";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &()).await?;
    Ok(count.try_into().unwrap_or(0))
}

pub async fn get_blog_category(id: Uuid, client: &Client) -> Result<Option<BlogCategory>, Error> {
    let q = "
    SELECT BlogCategory {
        id,
        title,
        slug
    } FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let cat: Option<BlogCategory> = client.query_single(q, &(id,)).await?;
    Ok(cat)
}

pub async fn get_blog_category_by_slug(slug: &str, client: &Client) -> Result<Option<BlogCategory>, Error> {
    let q = "
    SELECT BlogCategory {
        id,
        title,
        slug
    } FILTER .slug = <str>$0";
    tracing::debug!("To query: {}", q);
    let cat: Option<BlogCategory> = client.query_single(q, &(slug,)).await?;
    Ok(cat)
}

pub async fn get_previous_post(created_at: EDatetime, cat_slug: Option<String>, client: &Client) -> Result<Option<RawBlogPost>, Error> {
    let mut filter_lines = vec![
        ".created_at < <datetime>$created_at",
        ".is_published = true",
    ];
    let edatime = EValue::Datetime(created_at);
    let mut pairs = indexmap! {
        "created_at" => Some(edatime),
    };
    if let Some(slug) = cat_slug {
        filter_lines.push(".categories.slug = <str>$slug");
        pairs.insert("slug", Some(EValue::Str(slug)));
    }
    let filter_expr = filter_lines.join(" AND ");
    let args = edge_object_from_simple_pairs(pairs);

    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        excerpt,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {{
            id,
            title,
            slug,
        }},
    }}
    FILTER {filter_expr} ORDER BY .created_at DESC LIMIT 1");
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let post: Option<RawBlogPost> = client.query_single(&q, &args).await?;
    Ok(post)
}

pub async fn get_next_post(created_at: EDatetime, cat_slug: Option<String>, client: &Client) -> Result<Option<RawBlogPost>, Error> {
    let mut filter_lines = vec![
        ".created_at > <datetime>$created_at",
        ".is_published = true",
    ];
    let edatime = EValue::Datetime(created_at);
    let mut pairs = indexmap! {
        "created_at" => Some(edatime),
    };
    if let Some(slug) = cat_slug {
        filter_lines.push(".categories.slug = <str>$slug");
        pairs.insert("slug", Some(EValue::Str(slug)));
    }
    let filter_expr = filter_lines.join(" AND ");
    let args = edge_object_from_simple_pairs(pairs);

    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        excerpt,
        is_published,
        published_at,
        created_at,
        updated_at,
        categories: {{
            id,
            title,
            slug,
        }},
    }}
    FILTER {filter_expr} ORDER BY .created_at ASC LIMIT 1");
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let post: Option<RawBlogPost> = client.query_single(&q, &args).await?;
    Ok(post)
}

pub async fn get_mini_post_by_old_id(old_id: u32, client: &Client) -> Result<Option<MiniBlogPost>, Error> {
    let q = "SELECT BlogPost {id, title, slug, created_at} FILTER .old_id = <int32>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(q, &(old_id as i32,)).await?;
    Ok(post)
}
