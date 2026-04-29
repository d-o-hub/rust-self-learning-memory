import urllib.request
try:
    urllib.request.urlopen('http://127.0.0.1:4000/api/memory/initiate-recording', data=b'')
except Exception:
    pass
