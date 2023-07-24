from quanweb.common import db
from quanweb.models import ModelMixIn


class Presentation(ModelMixIn, db.Model):
    __tablename__ = 'presentations'
    title = db.Column(db.Unicode(400), nullable=False)
    url = db.Column(db.Unicode(400), nullable=False)
    event = db.Column(db.Unicode(200), nullable=True)

    def __str__(self):
        return self.title
