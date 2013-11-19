from .data import manager
from .serv import MyServer

# Override built-in command
manager.add_command('runserver', MyServer())
