# Global config

import os.path
from os.path import dirname, abspath

SITE_TITLE = "Quân's blog"
SITE_DESCRIPTION = 'A blog inspired by Wordpress'

PROJECT_FOLDER = dirname(dirname(abspath(__file__)))
STATIC_FOLDER = os.path.join(PROJECT_FOLDER, 'static')
TEMPLATE_FOLDER = os.path.join(PROJECT_FOLDER, 'templates')
TEMPLATES_AUTO_RELOAD = True
SQLALCHEMY_TRACK_MODIFICATIONS = False

SECRET_KEY = '{{secret_key}}'
SQLALCHEMY_DATABASE_URI = 'sqlite:////tmp/test.db'
SQLALCHEMY_ECHO = False

# Secret config saved in secret.py
try:
    from .secret import *  # noqa
except ImportError:
    pass
