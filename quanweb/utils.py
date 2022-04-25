def truncate_text(text, max_length=120):
    if not text:
        return ''
    if len(text) <= max_length:
        return text
    return text[:max_length-1] + '…'
