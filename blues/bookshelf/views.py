from flask import Blueprint
from quanweb import config

bookshelfm = Blueprint('bookshelf', __name__,
                       static_folder=config.STATIC_FOLDER,
                       template_folder=config.TEMPLATE_FOLDER)
