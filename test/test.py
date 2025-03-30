from bottle import route, run, request
import threading
import time
import requests
import queue
import os
import json

q = queue.SimpleQueue()
    

def test_post_trigger1():
    time.sleep(1)
    test_data = {"testxyz": 1}
    res = requests.post("http://localhost:7357/trigger1", json=test_data)

    assert(res.ok)

    r = q.get(timeout=2)
    print("RECEIVED FROM Q", r)
    assert r["method"] == "POST"
    assert r["path"] == "test1"
    assert r["body"] == test_data

def test_put_trigger2():
    time.sleep(1)
    test_data = {"testxyz": 2}
    res = requests.post("http://localhost:7357/trigger2", json=test_data)

    assert(res.ok)

    r = q.get(timeout=2)
    print("RECEIVED FROM Q", r)
    assert r["method"] == "PUT"
    assert r["path"] == "test2"
    assert r["body"] == test_data
    hh = r["headers"]

    # TODO: Why is apiKey reformatted to Apikey???
    secret = hh.get("Apikey", "")
    assert secret == "secret123"

def not_a_test_two_actions():
    # Trigger starts actionA and actionB
    # ActionA, type generate, generates a json body "jsonA"
    # ActionB, type http, receives json and posts it to testreceiver

    test_data = {"testxyz": 1}
    print ("TEST TRIGGER")
    res = requests.post("http://localhost:3000/test_two_actions", json=test_data)
    assert(res.ok)

    expected_body = {}
    # Statically generated data: {"test2": "generated"}
    r = q.get(timeout=2)
    print("RECEIVED FROM Q", r)
    assert r["method"] == "POST"
    assert r["path"] == "test2"
    assert r["body"] == test_data

def thread_function(name):
    run(host='localhost', port=8000)

x = threading.Thread(target=thread_function, args=(1,), daemon=True)
x.start()

# app = Bottle()
@route('/<path>', method=['POST', 'PUT'])
def index(path):
    print("Method ", request.method)
    body = json.load(request.body)
    print ("BODY ", body)
    print("REQ HEADERS", str(request.headers) )
    headers = {}
    for k in request.headers:
        print("REQ HEADERS", str(k), request.headers.get(k) )
        headers[k] = request.headers.get(k)
        # headers.append((k, request.headers.get(k)))

    q.put({"method":request.method, "path": path, "body": body, "headers": headers})

    return "OK"
