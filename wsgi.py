import logging
from logentries import LogentriesHandler
from waitress import serve

from quanweb import app
from quanweb.config import LOGENTRIES_TOKEN

ADDRESS = '127.0.0.1'
PORT = 2750

handler = LogentriesHandler(LOGENTRIES_TOKEN)
app.logger.addHandler(handler)
logging.getLogger('sqlalchemy').addHandler(handler)

if __name__ == '__main__':
    serve(app, host=ADDRESS, port=PORT)
