
from flask_wtf import Form
from wtforms.fields import PasswordField, SubmitField
from wtforms.fields.html5 import EmailField
from wtforms.validators import DataRequired

from quanweb.models import User


class LoginForm(Form):
    email = EmailField(validators=[DataRequired()])
    password = PasswordField(validators=[DataRequired()])
