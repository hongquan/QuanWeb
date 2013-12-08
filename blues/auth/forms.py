from flask.ext.wtf import Form
from wtforms.fields import PasswordField, SubmitField
from wtforms.fields.html5 import EmailField
from wtforms.validators import DataRequired

from .models import User


class LoginForm(Form):
    email = EmailField(validators=[DataRequired()])
    password = PasswordField(validators=[DataRequired()])
    submit = SubmitField()
