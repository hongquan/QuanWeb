from ..common import app
from .. import config

@app.context_processor
def inject_config():
    return {'config': config}
