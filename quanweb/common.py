from flask import Flask
from flask_sqlalchemy import SQLAlchemy
from flask_login import LoginManager
from flask_behind_proxy import FlaskBehindProxy

from . import config

# Constants
UNCATEGORIZED = '_uncategorized'

app = Flask(__name__, static_folder=config.STATIC_FOLDER,
            template_folder=config.TEMPLATE_FOLDER)
app.config.from_object(config)

db = SQLAlchemy()
loginmanager = LoginManager()

# Generate URL correctly if behind reverse proxy
proxied = FlaskBehindProxy(app)
