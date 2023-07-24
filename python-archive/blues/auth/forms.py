from flask_wtf import FlaskForm
from wtforms.fields import PasswordField, SubmitField
from wtforms.fields import EmailField
from wtforms.validators import DataRequired


class LoginForm(FlaskForm):
    email = EmailField(validators=[DataRequired()])
    password = PasswordField(validators=[DataRequired()])
    submit = SubmitField()
