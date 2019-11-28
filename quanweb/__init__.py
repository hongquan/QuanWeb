import sys
import os.path
from os.path import dirname

from flask_bootstrap import Bootstrap
from flask_admin import Admin

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
from talk import talkm

from .common import app, loginmanager, db
from . import views, widedata   # NOQA
from admini.views import AdminHomeView, CategoryAdmin, EntryAdmin, UserAdmin, PresentationAdmin

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
app.register_blueprint(talkm, url_prefix='/talk')

# Admin
admin = Admin(app, index_view=AdminHomeView(),
              base_template='admin/master_local.html',
              template_mode='bootstrap3')
admin.add_view(CategoryAdmin())
admin.add_view(EntryAdmin())
admin.add_view(UserAdmin())
admin.add_view(PresentationAdmin())
