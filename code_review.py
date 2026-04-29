import urllib.request
try:
    urllib.request.urlopen('http://127.0.0.1:4000/api/code-review/do-review', data=b'')
except Exception:
    pass
