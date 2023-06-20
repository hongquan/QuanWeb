import logging
from rich.logging import RichHandler

import click
import edgedb
from logbook import Logger, DEBUG
from chameleon_log.amend import StdLoggingHandler

from quanweb.common import db, app
from blues.auth.models import User


logger = Logger(__name__)
DB_NAME = 'quanweb'


@click.group()
def cli():
    pass


@cli.command()
def copy_to_edgedb():
    client = edgedb.create_client(database=DB_NAME)
    users = User.query.all()
    logger.info('users: {}', users)
    q = '''
    INSERT User {
        username := <str>$username,
        password := <str>$password,
        first_name := <optional str>$firstname,
        last_name := <optional str>$lastname,
        email := <str>$email,
        is_active := <bool>$active,
        is_superuser := <bool>$is_superuser,
    }
    '''
    for u in users:
        firstname = u.firstname
        lastname = u.lastname
        r = client.query(q, username=u.username, password=u._password,
                         firstname=firstname, lastname=lastname,
                         email=u.email, active=u.active, is_superuser=u.is_superuser)
        logger.info('r: {}', r)


if __name__ == '__main__':
    logging.basicConfig(handlers=[RichHandler()])
    StdLoggingHandler(level=DEBUG).push_application()
    db.init_app(app)
    with app.app_context():
        cli()
