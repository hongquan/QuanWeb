use uuid::Uuid;
use edgedb_tokio::{Client, Error};
use edgedb_protocol::model::Datetime as EDatetime;
use edgedb_protocol::value::Value as EValue;
use edgedb_protocol::common::Cardinality as Cd;
use indexmap::{indexmap, IndexMap};

use crate::models::{MediumBlogPost, DetailedBlogPost, BlogCategory, MiniBlogPost};
use crate::types::conversions::{edge_object_from_simple_pairs, edge_object_from_pairs};

pub async fn count_search_result_posts(lower_search_tokens: Option<&Vec<String>>, client: &Client) -> Result<usize, Error> {
    let lower_search_tokens: Option<Vec<&str>> = lower_search_tokens.map(|v| v.iter().map(|s| s.as_str()).collect());
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
        author: {id, username, email},
        seo_description,
        og_image,
    }
    FILTER .id = <uuid>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<DetailedBlogPost> = client.query_single(q, &(post_id,)).await?;
    Ok(post)
}

pub async fn get_detailed_post_by_slug(slug: String, client: &Client) -> Result<Option<DetailedBlogPost>, Error> {
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
        author: {id, username, email},
        seo_description,
        og_image,
    }
    FILTER .slug = <str>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<DetailedBlogPost> = client.query_single(q, &(slug,)).await?;
    Ok(post)
}

pub async fn get_blogposts(lower_search_tokens: Option<&Vec<String>>, offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<MediumBlogPost>, Error> {
    let filter_line = if lower_search_tokens.is_some() {
        "FILTER all(contains(str_lower(.title), array_unpack(<array<str>>$tokens)))"
    } else {
        ""
    };
    let mut pairs = IndexMap::with_capacity(3);
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    if let Some(ss) = lower_search_tokens {
        let search: Vec<EValue> = ss.into_iter().map(|s| EValue::Str(s.into())).collect();
        pairs.insert("tokens", (Some(EValue::Array(search)), Cd::One));
    }
    if let Some(offset) = offset {
        pairs.insert("offset", (Some(EValue::Int64(offset)), Cd::One));
        paging_lines.push(format!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        pairs.insert("limit", (Some(EValue::Int64(limit)), Cd::One));
        paging_lines.push(format!("LIMIT <int64>$limit"));
    }
    let paging_expr = paging_lines.join(" ");
    let args = edge_object_from_pairs(pairs);
    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        locale,
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
        author: {{
            id,
            username,
            email,
        }},
    }}
    {filter_line}
    ORDER BY .created_at DESC EMPTY FIRST {paging_expr}");
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:?}", args);
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn get_published_posts(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<MediumBlogPost>, Error> {
    let mut pairs = IndexMap::with_capacity(2);
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    if let Some(offset) = offset {
        pairs.insert("offset", (Some(EValue::Int64(offset)), Cd::One));
        paging_lines.push(format!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        pairs.insert("limit", (Some(EValue::Int64(limit)), Cd::One));
        paging_lines.push(format!("LIMIT <int64>$limit"));
    }
    let paging_expr = paging_lines.join(" ");
    let args = if pairs.is_empty() {
        EValue::Nothing
    } else {
        edge_object_from_pairs(pairs)
    };
    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        locale,
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
        author: {{
            id,
            username,
            email,
        }},
    }}
    FILTER .is_published = true ORDER BY .created_at DESC EMPTY FIRST {paging_expr}");
    tracing::info!("To query: {}", q);
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn get_published_posts_under_category(cat_slug: Option<String>, offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<MediumBlogPost>, Error> {
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
        locale,
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
        author: {{
            id,
            username,
            email,
        }},
    }}
    FILTER {filter_expr} ORDER BY .created_at DESC EMPTY FIRST {paging_expr}");
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

pub async fn get_published_uncategorized_blogposts(offset: Option<i64>, limit: Option<i64>, client: &Client) -> Result<Vec<MediumBlogPost>, Error> {
    let mut pairs = IndexMap::with_capacity(2);
    let mut paging_lines: Vec<String> = Vec::with_capacity(2);
    if let Some(offset) = offset {
        pairs.insert("offset", (Some(EValue::Int64(offset)), Cd::One));
        paging_lines.push(format!("OFFSET <int64>$offset"));
    }
    if let Some(limit) = limit {
        pairs.insert("limit", (Some(EValue::Int64(limit)), Cd::One));
        paging_lines.push(format!("LIMIT <int64>$limit"));
    }
    let paging_expr = paging_lines.join(" ");
    let args = edge_object_from_pairs(pairs);
    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        locale,
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
        author: {{
            id,
            username,
            email,
        }},
    }}
    FILTER .is_published = true AND NOT EXISTS .categories ORDER BY .created_at DESC EMPTY FIRST {paging_expr}");
    tracing::debug!("To query: {}", q);
    tracing::debug!("With args: {:#?}", args);
    let posts: Vec<MediumBlogPost> = client.query(&q, &args).await?;
    Ok(posts)
}

pub async fn count_published_uncategorized_posts(client: &Client) -> Result<usize, Error> {
    let q = "
    SELECT count((SELECT BlogPost FILTER .is_published = true AND NOT EXISTS .categories))";
    tracing::debug!("To query: {}", q);
    let count: i64 = client.query_required_single(q, &()).await?;
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

pub async fn get_category(id: Uuid, client: &Client) -> Result<Option<BlogCategory>, Error> {
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

pub async fn get_category_by_slug(slug: &str, client: &Client) -> Result<Option<BlogCategory>, Error> {
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

pub async fn get_previous_post(created_at: EDatetime, cat_slug: Option<&str>, client: &Client) -> Result<Option<MiniBlogPost>, Error> {
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
        pairs.insert("slug", Some(EValue::Str(slug.to_string())));
    }
    let filter_expr = filter_lines.join(" AND ");
    let args = edge_object_from_simple_pairs(pairs);

    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        created_at,
        updated_at,
    }}
    FILTER {filter_expr} ORDER BY .created_at DESC LIMIT 1");
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(&q, &args).await?;
    Ok(post)
}

pub async fn get_next_post(created_at: EDatetime, cat_slug: Option<&str>, client: &Client) -> Result<Option<MiniBlogPost>, Error> {
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
        pairs.insert("slug", Some(EValue::Str(slug.to_string())));
    }
    let filter_expr = filter_lines.join(" AND ");
    let args = edge_object_from_simple_pairs(pairs);

    let q = format!("
    SELECT BlogPost {{
        id,
        title,
        slug,
        created_at,
        updated_at,
    }}
    FILTER {filter_expr} ORDER BY .created_at ASC LIMIT 1");
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(&q, &args).await?;
    Ok(post)
}

pub async fn get_last_updated_post(client: &Client) -> Result<Option<MiniBlogPost>, Error> {
    let q = "
    SELECT BlogPost {
        id,
        title,
        slug,
        created_at,
        updated_at,
    } FILTER .is_published = true ORDER BY .updated_at DESC LIMIT 1";
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(q, &()).await?;
    Ok(post)
}

pub async fn get_mini_post_by_old_id(old_id: u32, client: &Client) -> Result<Option<MiniBlogPost>, Error> {
    let q = "SELECT BlogPost {id, title, slug, created_at, updated_at} FILTER .old_id = <int32>$0";
    tracing::debug!("To query: {}", q);
    let post: Option<MiniBlogPost> = client.query_single(q, &(old_id as i32,)).await?;
    Ok(post)
}
