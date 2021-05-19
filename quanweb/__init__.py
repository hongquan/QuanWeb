
from flask_bootstrap import Bootstrap
from flask_admin import Admin

# Blueprints
from blues.front import frontpage
from blues.blog import blogm
from blues.auth import authm
from blues.auth.models import AnonymousUser
from blues.bookshelf import bookshelfm
from blues.talk import talkm

from . import views, widedata   # NOQA
from .common import app, loginmanager, db
from blues.admini.views import AdminHomeView, CategoryAdmin, EntryAdmin, UserAdmin, PresentationAdmin

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
