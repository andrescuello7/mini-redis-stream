const net = require('node:net');
const http = require('node:http');

const HTTP_PORT = 3000;
const REDIS_HOST = '127.0.0.1';
const REDIS_PORT = 6379;

function encodeRESP(args) {
  let resp = `*${args.length}\r\n`;
  for (const arg of args) {
    resp += `$${Buffer.byteLength(arg)}\r\n${arg}\r\n`;
  }
  return resp;
}

function parseRESP(raw) {
  const str = raw.toString();
  if (str.startsWith('+')) return str.slice(1).trim();
  if (str.startsWith('-')) throw new Error(str.slice(1).trim());
  if (str.startsWith('$')) {
    const lines = str.split('\r\n');
    const len = Number.parseInt(lines[0].slice(1), 10);
    if (len === -1) return null;
    return lines[1];
  }
  return str.trim();
}

function redisCommand(args) {
  return new Promise((resolve, reject) => {
    const socket = net.createConnection({ host: REDIS_HOST, port: REDIS_PORT });
    let buffer = '';

    socket.on('connect', () => socket.write(encodeRESP(args)));

    socket.on('data', (data) => {
      buffer += data.toString();
      socket.destroy();
    });

    socket.on('close', () => {
      try {
        resolve(parseRESP(buffer));
      } catch (err) {
        reject(err);
      }
    });

    socket.on('error', reject);
  });
}

function readBody(req) {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', chunk => (body += chunk));
    req.on('end', () => {
      try {
        resolve(JSON.parse(body));
      } catch {
        reject(new Error('Invalid JSON body'));
      }
    });
    req.on('error', reject);
  });
}

const server = http.createServer(async (req, res) => {
  const url = new URL(req.url, `http://${req.headers.host}`);
  res.setHeader('Content-Type', 'application/json');

  try {
    if (req.method === 'GET' && url.pathname === '/get') {
      const key = url.searchParams.get('key');
      if (!key) {
        res.writeHead(400);
        return res.end(JSON.stringify({ error: 'key query param is required' }));
      }
      const value = await redisCommand(['GET', key]);
      res.writeHead(200);
      res.end(JSON.stringify({ key, value }));

    } else if (req.method === 'POST' && url.pathname === '/set') {
      const body = await readBody(req);
      const { key, value } = body;
      if (!key || value === undefined) {
        res.writeHead(400);
        return res.end(JSON.stringify({ error: 'key and value are required in body' }));
      }
      const reply = await redisCommand(['SET', key, String(value)]);
      res.writeHead(200);
      res.end(JSON.stringify({ ok: true, reply }));

    } else {
      res.writeHead(404);
      res.end(JSON.stringify({ error: 'Not found' }));
    }
  } catch (err) {
    res.writeHead(500);
    res.end(JSON.stringify({ error: err.message }));
  }
});

server.listen(HTTP_PORT, () => {
  console.log(`HTTP API → http://127.0.0.1:${HTTP_PORT}`);
  console.log('  GET  /get?key=<key>');
  console.log('  POST /set   body: { "key": "...", "value": "..." }');
});
