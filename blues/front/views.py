from flask import Blueprint
from flask import request, render_template, abort
from jinja2 import TemplateNotFound

from quanweb import config
from blog.models import Entry

frontpage = Blueprint('frontpage', __name__, static_folder=config.STATIC_FOLDER,
                       template_folder=config.TEMPLATE_FOLDER)

@frontpage.route('/')
def index():
    posts = Entry.pub().order_by(Entry.date_published.desc())
    page = int(request.args.get('page', 1))
    ctx = {
        'posts': posts.all(),
        'pagination': posts.paginate(page, 5),
        'endpoint': 'frontpage.index',
    }
    return render_template('front/index.html', **ctx)
