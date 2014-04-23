import os.path
import gzip


def import_file(filepath):
    base, ext = os.path.splitext(filepath)
    if ext == '.gz':
        with gzip.open(filepath, 'rb') as fl:
            content = fl.read()
    elif ext in ('.md', ''):
        with open(filepath) as fl:
            content = fl.read()


def truncate_text(text, max_length=120):
    if not text:
        return ''
    if len(text) <= max_length:
        return text
    return text[:max_length-1] + '…'