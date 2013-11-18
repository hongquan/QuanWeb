from flask import Flask
from flask.ext.sqlalchemy import SQLAlchemy
from flask.ext.login import LoginManager
from flask.ext.markdown import Markdown

from . import config

app = Flask(__name__, static_folder=config.STATIC_FOLDER,
            template_folder=config.TEMPLATE_FOLDER)
app.config.from_object(config)

db = SQLAlchemy()
loginmanager = LoginManager()

loginmanager.login_view = 'views.login'


# Markdown
md = Markdown(app, extensions=('fenced_code', 'codehilite(linenums=True)'),
              safe_mode=True, output_format='html5')
