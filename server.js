const net = require('net');
const readline = require('readline');

const socket = net.createConnection({
  host: '127.0.0.1',
  port: 6379
});

socket.on('connect', () => {
  console.log('Connected to Redis server');
});

socket.on('data', (data) => {
  console.log('\nSERVER RESPONSE:');
  console.log(data.toString());
});

socket.on('close', () => {
  console.log('Connection closed');
});

socket.on('error', (err) => {
  console.error(err);
});

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

function encodeRESP(input) {
  const parts = input.trim().split(' ');

  let resp = `*${parts.length}\r\n`;
  // get -> b"*2\r\n$3\r\nget\r\n$3\r\nfoo\r\n"
  // get -> b"*2\r\n$3\r\nget\r\n$3\r\nfoo\r\n"

  // set -> b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n"
  // set -> b"*3\r\n$3\r\nhello\r\n$5\r\nworld\r\n"
  
  resp += `$3\r\nset$3\r\n`; // Command is always 3 bytes for simplicity
  for (const part of parts) {
    // resp += `$${Buffer.byteLength(part)}\r\n`;
    resp += `${part}\r\n`;
  }

  return resp;
}

rl.on('line', (line) => {
  const resp = encodeRESP(line);

  console.log('\nSENDING RESP:');
  console.log(JSON.stringify(resp));

  socket.write(resp);
});