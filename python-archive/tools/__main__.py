import os
import logging
from datetime import datetime
from zoneinfo import ZoneInfo
from typing import cast

import click
import edgedb
import hashers
from logbook import Logger, DEBUG
from rich.logging import RichHandler
from chameleon_log.amend import StdLoggingHandler
from sqlalchemy.orm import joinedload

from quanweb.common import db, app
from blues.auth.models import User
from blues.blog.models import Category, Entry
from blues.bookshelf.models import Author, Book
from blues.talk.models import Presentation


logger = Logger(__name__)
DB_NAME = 'quanweb'
EDGEDB_INSTANCE = 'QuanWeb'
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


def copy_authors(client: edgedb.Client):
    authors = db.session.query(Author).order_by('id')
    q = '''
    SELECT (
        INSERT BookAuthor {
            name := <str>$name,
            old_id := <int16>$id,
        } UNLESS CONFLICT ON .old_id ELSE (
            UPDATE BookAuthor SET {
                name := <str>$name,
            }
        )
    ) {
        name,
    }
    '''
    for u in authors:
        r = client.query(q, name=u.name, id=u.id)
        logger.info('r: {}', r)


def copy_books(client: edgedb.Client):
    books = db.session.query(Book).order_by('id')
    q = '''
    SELECT (
        INSERT Book {
            title := <str>$title,
            download_url := <optional str>$download_url,
            author := (
                SELECT BookAuthor FILTER .old_id = <optional int16>$old_author_id
            ),
            created_at := <datetime>$created_at,
            updated_at := <optional datetime>$updated_at,
            created_by := (
                SELECT User FILTER .old_id = <optional int16>$old_user_id
            ),
            old_id := <int16>$id,
        } UNLESS CONFLICT ON .old_id ELSE (
            UPDATE Book SET {
                title := <str>$title,
                download_url := <optional str>$download_url,
                author := (
                    SELECT BookAuthor FILTER .old_id = <optional int16>$old_author_id
                ),
                created_at := <datetime>$created_at,
                updated_at := <optional datetime>$updated_at,
                created_by := (
                    SELECT User FILTER .old_id = <optional int16>$old_user_id
                ),
            }
        )
    ) { title }
    '''
    for b in books:
        created_at = cast(datetime | None, b.date_created)
        if created_at and not created_at.tzinfo:
            created_at = created_at.astimezone(TZ_VN)
        updated_at = cast(datetime | None, b.date_modified)
        if updated_at and not updated_at.tzinfo:
            updated_at = updated_at.astimezone(TZ_VN)
        input_data = dict(title=b.title, download_url=b.download_url,
                          old_author_id=b.author_id,
                          created_at=created_at, updated_at=updated_at,
                          old_user_id=b.user_id, id=b.id)
        logger.info('Create Book with: {}', input_data)
        r = client.query(q, **input_data)
        logger.info('r: {}', r)


def copy_presentations(client: edgedb.Client):
    presentations = db.session.query(Presentation).order_by('id')
    q = '''
    SELECT (
        INSERT Presentation {
            title := <str>$title,
            url := <str>$url,
            old_id := <int16>$id,
        } UNLESS CONFLICT ON .old_id ELSE (
            UPDATE Presentation SET {
                title := <str>$title,
                url := <str>$url,
            }
        )
    ) { title }
    '''
    for p in presentations:
        r = client.query(q, title=p.title, url=p.url, id=p.id)
        logger.info('r: {}', r)


@cli.command()
def copy_to_edgedb():
    client = edgedb.create_client(EDGEDB_INSTANCE, database=DB_NAME).with_config()
    copy_users(client)
    copy_categories(client)
    copy_posts(client)
    copy_authors(client)
    copy_books(client)
    copy_presentations(client)


@cli.command()
@click.option('-e', '--email', prompt='Email', help='Email')
@click.option('-p', '--password', prompt='Password', help='Password')
def set_user_password_in_edgedb(email: str, password: str):
    client = edgedb.create_client(EDGEDB_INSTANCE, database=DB_NAME)
    q = '''
    UPDATE User
    FILTER .email = <str>$email
    SET {
        password := <str>$password,
    }
    '''
    hashed_password = hashers.hashpw(password, "argon2")
    logger.info('Hashed_password: {}', hashed_password)
    r = client.query(q, email=email, password=hashed_password)
    logger.info('r: {}', r)


if __name__ == '__main__':
    logging.basicConfig(format='%(message)s', handlers=[RichHandler()])
    StdLoggingHandler(level=DEBUG).push_application()
    db.init_app(app)
    os.environ.setdefault('EDGEDB_DEBUG_SERVER', '1')
    with app.app_context():
        cli()
