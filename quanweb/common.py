from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_login import LoginManager
from flaskext.markdown import Markdown
from flask_reverse_proxy import FlaskReverseProxied

from . import config

# Constants
UNCATEGORIZED = '_uncategorized'

app = Flask(__name__, static_folder=config.STATIC_FOLDER,
            template_folder=config.TEMPLATE_FOLDER)
app.config.from_object(config)

db = SQLAlchemy()
loginmanager = LoginManager()


# Markdown
md = Markdown(app, extensions=('fenced_code', 'codehilite',
                               'mdx_linkify',
                               'markdown.extensions.tables',
                               'markdown.extensions.attr_list'),
              safe_mode=True, output_format='html5')

# Generate URL correctly if behind reverse proxy
proxied = FlaskReverseProxied(app)
