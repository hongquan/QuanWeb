from flask import Blueprint
from flask import request, render_template
from flask_login import current_user

from quanweb import config
from blues.blog.models import Entry

PER_PAGE = 5

frontpage = Blueprint('frontpage', __name__, static_folder=config.STATIC_FOLDER,
                      template_folder=config.TEMPLATE_FOLDER)


@frontpage.route('/')
def index():
    posts = Entry.pub().order_by(Entry.date_published.desc())
    page = int(request.args.get('page', 1))
    start = (page - 1) * PER_PAGE
    end = start + PER_PAGE
    ctx = {
        'posts': posts.slice(start, end),
        'pagination': posts.paginate(page, PER_PAGE),
        'endpoint': 'frontpage.index',
        'no_tracking': current_user.is_authenticated
    }
    return render_template('front/index.jinja', **ctx)
