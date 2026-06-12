import http.server, socketserver, os, time, subprocess, threading, json

PORT = 8000
SRC_DIR = 'src'
BUILD_TS = [0.0]

def rebuild():
    result = subprocess.run(
        ['wasm-pack', 'build', '--target', 'web', '--out-dir', 'pkg'],
        capture_output=True, text=True
    )
    BUILD_TS[0] = time.time()
    if result.returncode == 0:
        print('  rebuilt ok')
    else:
        print('  rebuild failed:\n' + result.stderr)

def watcher():
    mtimes = {}
    while True:
        changed = False
        try:
            for fname in os.listdir(SRC_DIR):
                if not fname.endswith('.rs'):
                    continue
                path = os.path.join(SRC_DIR, fname)
                mt = os.path.getmtime(path)
                if fname in mtimes:
                    if mt > mtimes[fname]:
                        mtimes[fname] = mt
                        changed = True
                else:
                    mtimes[fname] = mt
        except FileNotFoundError:
            pass
        if changed:
            rebuild()
        time.sleep(1)

class Handler(http.server.SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/build-ts':
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Cache-Control', 'no-cache')
            self.end_headers()
            self.wfile.write(json.dumps({'ts': BUILD_TS[0]}).encode())
        else:
            super().do_GET()

    def log_message(self, format, *args):
        if args[0] != '/build-ts':
            super().log_message(format, *args)

if __name__ == '__main__':
    print('Initial build...')
    rebuild()
    threading.Thread(target=watcher, daemon=True).start()
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(('', PORT), Handler) as httpd:
        print(f'Serving on http://localhost:{PORT}')
        httpd.serve_forever()
