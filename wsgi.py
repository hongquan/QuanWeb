import logging
#from logentries import LogentriesHandler
from waitress import serve
import appenlight_client.ext.flask as appenlight

from quanweb import app
from quanweb.config import APPENLIGHT_PRVKEY

ADDRESS = '127.0.0.1'
PORT = 2750

#handler = LogentriesHandler(LOGENTRIES_TOKEN)
#app.logger.addHandler(handler)
#logging.getLogger('sqlalchemy').addHandler(handler)
app = appenlight.add_appenlight(app, {'appenlight.api_key': APPENLIGHT_PRVKEY})

if __name__ == '__main__':
    serve(app, host=ADDRESS, port=PORT)
