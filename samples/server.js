const http = require('http');
const net = require('net');

const PORT = process.env.PORT || 8080;
const BACKEND_NAME = `Backend-${PORT}`;
const SOCKET_HOST = process.env.SOCKET_HOST || '127.0.0.1';
const SOCKET_PORT = Number(process.env.SOCKET_PORT || 6379);
const SOCKET_TIMEOUT_MS = Number(process.env.SOCKET_TIMEOUT_MS || 1000);

function probeSocket() {
    return new Promise((resolve) => {
        const startedAt = Date.now();
        const socket = new net.Socket();
        let settled = false;

        const finish = (result) => {
            if (settled) {
                return;
            }

            settled = true;
            socket.destroy();
            resolve({
                host: SOCKET_HOST,
                port: SOCKET_PORT,
                elapsedMs: Date.now() - startedAt,
                ...result
            });
        };

        socket.setTimeout(SOCKET_TIMEOUT_MS);
        socket.once('connect', () => finish({ ok: true }));
        socket.once('timeout', () => finish({ ok: false, error: `timeout after ${SOCKET_TIMEOUT_MS}ms` }));
        socket.once('error', (error) => finish({ ok: false, error: error.message }));
        socket.connect(SOCKET_PORT, SOCKET_HOST);
    });
}

const server = http.createServer(async (req, res) => {
    console.log(`[${BACKEND_NAME}] ${req.method} ${req.url} - ${new Date().toISOString()}`);

    if (req.url === '/health') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ status: 'healthy', backend: BACKEND_NAME }));
        return;
    }

    if (req.url === '/socket-test') {
        const probe = await probeSocket();
        res.writeHead(probe.ok ? 200 : 503, {
            'Content-Type': 'application/json',
            'X-Backend-Server': BACKEND_NAME
        });
        res.end(JSON.stringify({
            backend: BACKEND_NAME,
            timestamp: new Date().toISOString(),
            socket: probe
        }, null, 2));
        return;
    }
    
    const response = {
        backend: BACKEND_NAME,
        port: PORT,
        timestamp: new Date().toISOString(),
        method: req.method,
        url: req.url,
        headers: req.headers,
        message: `Response from ${BACKEND_NAME} on port ${PORT}`
    };

    res.writeHead(200, { 
        'Content-Type': 'application/json',
        'X-Backend-Server': BACKEND_NAME
    });
    res.end(JSON.stringify(response, null, 2));
});

server.listen(PORT, () => {
    console.log(`✓ ${BACKEND_NAME} listening on http://127.0.0.1:${PORT}`);
});