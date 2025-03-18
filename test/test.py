from bottle import route, run, post
import threading
import time
import requests
import queue
import os

q = queue.SimpleQueue()

def test_trigger1() -> bool:
    time.sleep(2)
    try:
        res = requests.post("http://localhost:3000/trigger1")
    except Exception as ex:
        print("Connection error. Is target running?", ex)
        return False
    
    if not res.ok:
         print("Failed to send trigger", res)
         return False
    try:
        r = q.get(timeout=2)
    except queue.Empty as e:
        print("No response", e)
        return False

    print("RECEIVED FROM Q", r)
    assert r["method"] == "POST"
    assert r["path"] == "test1"
    return res.ok

def thread_function(name):

    result = test_trigger1()

    print("Test result:", result)
    os._exit(0)

#     print("NAME ", name)
#     time.sleep(5)
#     res = requests.post("http://localhost:3000/trigger1")
#     r = q.get()
#     print("RECEIVED FROM Q", r)
#     assert r["method"] == "POST"
#     assert r["path"] == "test1"
#     if res.ok:
#         print("RES OK, NOW EXIT!!!")
# #           sys.exit() #  stderr.close()
#         break


x = threading.Thread(target=thread_function, args=(1,), daemon=True)
x.start()

@post('/<path>')
def index(path):
    print("POST", path)
    q.put({"method":"POST", "path": path})

    return "OK"

run(host='localhost', port=8000)

x.join()

print("DONE!!")
