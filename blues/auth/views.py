from flask import Blueprint, request
from flask import render_template, redirect, flash, url_for
from flask.views import MethodView
from flask_login import login_user, logout_user

from quanweb import config
from quanweb.common import loginmanager

from .forms import LoginForm
from .models import User
from .models import User


authm = Blueprint('auth', __name__, static_folder=config.STATIC_FOLDER,
                  template_folder=config.TEMPLATE_FOLDER)


class LoginView(MethodView):
    def get(self):
        form = LoginForm()
        return render_template('auth/login.html', form=form)

    def post(self):
        form = LoginForm()
        if form.validate_on_submit():
            print('Validated')
            user = User.query.authenticate(form.email.data, form.password.data)
            if user:
                flash('Login successfully', 'success')
                login_user(user)
                return redirect('/')
            # else
            flash('Wrong data', 'error')

        return redirect(url_for('login'))



class LogoutView(MethodView):
    def get(self):
        logout_user()
        flash('You logged out', 'info')
        return redirect('/')




@loginmanager.user_loader
def load_user(userid):
    return User.get(userid)
