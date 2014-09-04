# Global config

import os.path
from os.path import dirname

SITE_TITLE = "Quân's blog"
SITE_DESCRIPTION = 'A blog inspired by Wordpress'

STATIC_FOLDER = os.path.join(dirname(dirname(__file__)), 'static')
TEMPLATE_FOLDER = os.path.join(dirname(dirname(__file__)), 'templates')
BOOTSTRAP_SERVE_LOCAL = True

SECRET_KEY = '{{secret_key}}'
SQLALCHEMY_DATABASE_URI = 'sqlite:////tmp/test.db'
SQLALCHEMY_ECHO = False

# logentries.com service
LOGENTRIES_TOKEN ='zzzz'

# App Enlight service
APPENLIGHT_PRVKEY = 'zzzz'

# Secret config saved in secret.py
try:
    from .secret import *
except ImportError:
    pass
