from flask import request
from ..common import app, UNCATEGORIZED
from .. import config

@app.context_processor
def inject_config():
    return {'config': config}

@app.context_processor
def is_running_locally():
    try:
        hostname, port = request.host.split(':')
        if port == '5000' or 'hostname' in ('localhost', '127.0.0.1'):
            return {'running_locally': True}
    except ValueError:
        pass
    return {}


@app.context_processor
def inject_constants():
    return {'UNCATEGORIZED': UNCATEGORIZED}
