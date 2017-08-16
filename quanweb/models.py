from .common import db


class ModelMixIn:
    id = db.Column(db.Integer, primary_key=True)

    def __repr__(self):
        # Convert <Model object at 0xAAAA> to <Model(string)>
        orig = super().__repr__()
        text = self.__str__()
        if not ' object at 0x' in orig:
            return orig
        pos = orig.find(' object at 0x')
        out = orig[:pos] + '({})'.format(text) + '>'
        return out
