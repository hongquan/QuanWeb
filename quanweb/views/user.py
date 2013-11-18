from flask import request, render_template
from ..forms import LoginForm

def login():
    form = LoginForm(next=request.args.get('next'))
    return render_template('login.html', form=form)
