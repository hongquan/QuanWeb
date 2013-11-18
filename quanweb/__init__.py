import sys
import os.path
from os.path import dirname

from flask import Flask, render_template, g, redirect, url_for, flash, session
from flask_bootstrap import Bootstrap

# Insert blueprint folder to PYTHONPATH
sys.path.insert(0, os.path.join(dirname(dirname(__file__)), 'blues'))

# Blueprints
from front import frontpage
from blog import blogm

import blog.models

from . import config
from .common import app, loginmanager
from . import views, filters, widedata
from .models import db, User


# SQLAlchemy
db.init_app(app)
loginmanager.init_app(app)

# Jinja
app.jinja_env.add_extension('jinja2.ext.i18n')

# Twitter Bootstrap
Bootstrap(app)

# Register Blueprints
app.register_blueprint(frontpage)
app.register_blueprint(blogm, url_prefix='/blog')

# Add app's vies
app.add_url_rule('/login', 'login', views.login)
