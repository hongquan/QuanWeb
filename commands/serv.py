from flask_script import Server

from .data import app

class MyServer(Server):
    ''' Our subclass, to set app.debug = True before running app '''
    def handle(self, app, host, port, use_debugger, use_reloader,
               threaded, processes, passthrough_errors):
        app.debug = True
        super().handle(app, host, port, use_debugger, use_reloader,
                       threaded, processes, passthrough_errors)
