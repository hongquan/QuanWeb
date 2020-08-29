from flask import Blueprint
from flask import render_template

from quanweb import config
from .models import Presentation


talkm = Blueprint('talk', __name__, static_folder=config.STATIC_FOLDER,
                  template_folder=config.TEMPLATE_FOLDER)


@talkm.route('/')
def list_presentations():
    presentations = Presentation.query.all()
    return render_template('talk/list.jinja', presentations=presentations)
