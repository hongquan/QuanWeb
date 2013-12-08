"""
    setup.py
    ~~~~~~~~

    :copyright: (c) 2010 by Dan Jacob.
    :license: BSD, see LICENSE for more details.
"""

"""
QuanWeb
--------

"""
from setuptools import setup

setup(
    name='quanweb',
    version='0.1',
    url='http://quan.hoabinh.vn',
    license='BSD',
    author='Nguyễn Hồng Quân',
    author_email='ng.hong.quan@gmail.com',
    description='My personal web',
    long_description=__doc__,
    packages=['quanweb'],
    zip_safe=False,
    test_loader='attest:Loader',
    test_suite='tests.suite',
    platforms='any',
    install_requires=[
        'Flask',
        'psycopg2',
        'Flask-SQLAlchemy',
        'Flask-WTF',
        'Flask-Login',
        'Flask-Script',
        'Flask-Markdown',
        'Flask-Bootstrap',
        'Pygments',
        'python-slugify',
        'cherrypy',
        'logentries',
    ],
    include_package_data=True
)
