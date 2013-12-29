import sys
import os.path
from os.path import dirname

from flask import Flask, render_template, g, redirect, url_for, flash, session
from flask_bootstrap import Bootstrap

# Insert blueprint folder to PYTHONPATH
_bluefolder = os.path.join(dirname(dirname(__file__)), 'blues')
if _bluefolder not in sys.path:
    sys.path.insert(1, _bluefolder)

# Blueprints
from front import frontpage
from blog import blogm
from auth import authm
from auth.models import AnonymousUser
from bookshelf import bookshelfm

import blog.models

from . import config
from .common import app, loginmanager, db
from . import views, widedata

# SQLAlchemy
db.init_app(app)
loginmanager.anonymous_user = AnonymousUser
loginmanager.init_app(app)
Bootstrap(app)

# Jinja
app.jinja_env.add_extension('jinja2.ext.i18n')


# Register Blueprints
app.register_blueprint(frontpage)
app.register_blueprint(blogm, url_prefix='/blog')
app.register_blueprint(authm, url_prefix='/auth')
app.register_blueprint(bookshelfm, url_prefix='/book')
