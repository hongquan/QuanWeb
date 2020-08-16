#!python3
import mistletoe

from .pygments_renderer import PygmentsRenderer


def make_excerpt(fullmarkdown):
    lines = fullmarkdown.splitlines(True)[:7]
    # Count "code block" marker (```)
    count = sum(1 for li in lines if li.startswith('```'))
    if (count % 2) == 1:  # There are odd number of marks
        if lines[-1].startswith('```'):
            # Remove last mark...
            lines = lines[:-1]
        else:
            # ...Or add another mark to make sure the number is even
            lines.append('```')
    reduced = ''.join(lines)
    return mistletoe.markdown(reduced) + '...'


def make_html(fullmarkdown):
    return mistletoe.markdown(fullmarkdown, PygmentsRenderer)
