import logging
from rich.logging import RichHandler

import click
import edgedb
from logbook import Logger, DEBUG
from chameleon_log.amend import StdLoggingHandler

from quanweb.common import db, app
from blues.auth.models import User
from blues.blog.models import Category


logger = Logger(__name__)
DB_NAME = 'quanweb'


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
        } UNLESS CONFLICT ON .email ELSE (
            UPDATE User SET {
                username := <str>$username,
                password := <str>$password,
                first_name := <optional str>$firstname,
                last_name := <optional str>$lastname,
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
                         email=u.email, active=u.active, is_superuser=u.is_superuser)
        logger.info('r: {}', r)


def copy_categories(client: edgedb.Client):
    categories = Category.query.all()
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


@cli.command()
def copy_to_edgedb():
    client = edgedb.create_client(database=DB_NAME)
    copy_users(client)
    copy_categories(client)


if __name__ == '__main__':
    logging.basicConfig(format='%(message)s', handlers=[RichHandler()])
    StdLoggingHandler(level=DEBUG).push_application()
    db.init_app(app)
    with app.app_context():
        cli()
