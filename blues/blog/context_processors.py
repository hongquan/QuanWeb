from .views import blogm, UNCATEGORIZED

@blogm.context_processor
def inject_constants():
    return {'UNCATEGORIZED': UNCATEGORIZED}
