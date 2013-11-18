from flask import url_for

from .common import app


app.jinja_env.globals['static'] = lambda filename: url_for('static', filename=filename)

@app.template_filter()
def strftime(date, fmt):
    return date.strftime(fmt)
