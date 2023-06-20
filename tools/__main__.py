import os
import logging
from datetime import datetime
from zoneinfo import ZoneInfo
from typing import cast

import click
import edgedb
from logbook import Logger, DEBUG
from rich.logging import RichHandler
from chameleon_log.amend import StdLoggingHandler
from sqlalchemy.orm import joinedload

from quanweb.common import db, app
from blues.auth.models import User
from blues.blog.models import Category, Entry


logger = Logger(__name__)
DB_NAME = 'quanweb'
TZ_VN = ZoneInfo('Asia/Ho_Chi_Minh')


@click.group()
def cli():
    pass


def copy_users(client: edgedb.Client):
    users = User.query.all()
    logger.info('users: {}', users)
    q = '''
    SELECT (
        INSERT User {
            username := <str>$username,
            password := <str>$password,
            first_name := <optional str>$firstname,
            last_name := <optional str>$lastname,
            email := <str>$email,
            is_active := <bool>$active,
            is_superuser := <bool>$is_superuser,
            old_id := <int16>$id,
        } UNLESS CONFLICT ON .old_id ELSE (
            UPDATE User SET {
                username := <str>$username,
                password := <str>$password,
                first_name := <optional str>$firstname,
                last_name := <optional str>$lastname,
                email := <str>$email,
                is_active := <bool>$active,
                is_superuser := <bool>$is_superuser,
            }
        )
    ) {
        id,
        email,
    }
    '''
    for u in users:
        firstname = u.firstname
        lastname = u.lastname
        r = client.query(q, username=u.username, password=u._password,
                         firstname=firstname, lastname=lastname,
                         email=u.email, active=u.active, is_superuser=u.is_superuser,
                         id=u.id)
        logger.info('r: {}', r)


def copy_categories(client: edgedb.Client):
    categories = Category.query.order_by('id')
    q = '''
    SELECT (
        INSERT BlogCategory {
            title:= <str>$title,
            slug := <str>$slug,
            old_id := <int16>$id,
        } UNLESS CONFLICT ON .old_id ELSE (
            UPDATE BlogCategory SET {
                title := <str>$title,
            slug := <str>$slug,
            }
        )
    ) {
        title,
        slug,
    }
    '''
    for cat in categories:
        r = client.query(q, title=cat.title, slug=cat.slug, id=cat.id)
        logger.info('r: {}', r)


def copy_posts(client: edgedb.Client):
    q = '''
    SELECT (
        INSERT BlogPost {
            title := <str>$title,
            slug := <str>$slug,
            body := <optional str>$body,
            format := <optional DocFormat>$format,
            locale := <optional str>$locale,
            excerpt := <optional str>$excerpt,
            html := <optional str>$html,
            is_published := <bool>$is_published,
            published_at := <datetime>$published_at,
            seo_description := <optional str>$seo_description,
            seo_keywords := array_unpack(<array<str>>$seo_keywords),
            og_image := <optional str>$og_image,
            created_at := <datetime>$created_at,
            updated_at := <optional datetime>$updated_at,
            author := (
                SELECT User FILTER .old_id = <optional int16>$old_author_id
            ),
            categories := (
                SELECT BlogCategory FILTER .old_id IN array_unpack(<array<int16>>$old_category_ids)
            ),
            old_id := <int16>$id,
        } UNLESS CONFLICT ON .old_id ELSE (
            UPDATE BlogPost SET {
                title := <str>$title,
                slug := <str>$slug,
                body := <optional str>$body,
                format := <optional DocFormat>$format,
                locale := <optional str>$locale,
                excerpt := <optional str>$excerpt,
                html := <optional str>$html,
                is_published := <bool>$is_published,
                published_at := <datetime>$published_at,
                seo_description := <optional str>$seo_description,
                seo_keywords := array_unpack(<array<str>>$seo_keywords),
                og_image := <optional str>$og_image,
                created_at := <datetime>$created_at,
                updated_at := <optional datetime>$updated_at,
                author := (
                    SELECT User FILTER .old_id = <optional int16>$old_author_id
                ),
                categories := (
                    SELECT BlogCategory FILTER .old_id IN array_unpack(<array<int16>>$old_category_ids)
                ),
            }
        )
    ) {
        title,
        slug,
    }
    '''
    entries = db.session.query(Entry).options(joinedload(Entry.categories)).order_by('id')
    for post in entries:
        locale = str(post.locale) if post.locale else None
        doc_format = 'Rst' if post.format == 'rst' else 'Md'
        old_author_id = post.author_id
        old_category_ids = [c.id for c in post.categories]
        seo_keywords = [k.strip() for k in post.seo_keywords.split(',')] if post.seo_keywords else []
        created_at = cast(datetime | None, post.date_created)
        if created_at and not created_at.tzinfo:
            created_at = created_at.astimezone(TZ_VN)
        updated_at = cast(datetime | None, post.date_modified)
        if updated_at and not updated_at.tzinfo:
            updated_at = updated_at.astimezone(TZ_VN)
        published_at = cast(datetime | None, post.date_published)
        if published_at and not published_at.tzinfo:
            published_at = published_at.astimezone(TZ_VN)
        input_data = dict(title=post.title, slug=post.slug, body=post.body,
                          format=doc_format, locale=locale, excerpt=post.excerpt,
                          html=post.html, is_published=post.published,
                          published_at=published_at, seo_description=post.seo_description,
                          seo_keywords=seo_keywords, og_image=post.og_image,
                          created_at=created_at, updated_at=updated_at,
                          old_author_id=old_author_id, old_category_ids=old_category_ids, id=post.id)
        logger.info('Create Post with: {}', input_data)
        r = client.query(q, **input_data)
        logger.info('r: {}', r)


@cli.command()
def copy_to_edgedb():
    client = edgedb.create_client(database=DB_NAME)
    copy_users(client)
    copy_categories(client)
    copy_posts(client)


if __name__ == '__main__':
    logging.basicConfig(format='%(message)s', handlers=[RichHandler()])
    StdLoggingHandler(level=DEBUG).push_application()
    db.init_app(app)
    os.environ.setdefault('EDGEDB_DEBUG_SERVER', '1')
    with app.app_context():
        cli()
