import os
import tempfile
from zipfile import ZipFile

def split_content(filepath):
    with open(filepath) as fl:
        content = fl.read()
    # Read first line to find title. To specify title,
    # first line should starts with "#" then title
    title = None
    pos = content.find('\n')
    if pos != -1:
        firstline = content[:pos]
        if firstline.startswith('#'):
            title = firstline[1:].strip()
            # Content will be the rest of file
            content = content[pos+1:]
    # If there is no title in file, use file name as title
    if not title:
        title = os.path.basename(base)
    return title, content


def import_zip(filepath):
    with ZipFile(filepath) as z:
        if not 'post.md' in z.namelist():
            return
        wdir = tempfile.mkdtemp()
        z.extractall(wdir)
        title, content = split_content(os.path.join(wdir, 'post.md'))
    os.rmtree(wdir)