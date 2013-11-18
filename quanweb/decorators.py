import functools

from flask import g, request, redirect, url_for, flash


def login_required(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):

        if g.user:
            return func(*args, **kwargs)

        flash("You are not allowed to look at this page", "error")
        return redirect(url_for("main.login", next=request.path))

    return wrapper
