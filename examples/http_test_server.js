// Tiny HTTP test server for examples/phase17_http_test.swami
// Run: D:\Softwares\NodeJS\node.exe examples\http_test_server.js
const http = require('http');

const srv = http.createServer((req, res) => {
  if (req.method === 'GET' && req.url === '/namaste') {
    const body = Buffer.from('नमस्ते दुनिया', 'utf8');
    res.writeHead(200, {
      'Content-Type': 'text/plain; charset=utf-8',
      'Content-Length': body.length,
      'X-Lipi': 'haan',
    });
    res.end(body);
  } else if (req.method === 'POST' && req.url === '/echo') {
    const chunks = [];
    req.on('data', (c) => chunks.push(c));
    req.on('end', () => {
      const body = Buffer.concat(chunks);
      res.writeHead(200, {
        'Content-Type': 'application/json',
        'Content-Length': body.length,
      });
      res.end(body);
    });
  } else if (req.method === 'GET' && req.url === '/chunked') {
    // No Content-Length → Node sends Transfer-Encoding: chunked
    res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });
    res.write('खंड-एक ');
    res.write('खंड-दो ');
    setTimeout(() => res.end('खंड-तीन'), 30);
  } else {
    const body = Buffer.from('नहीं मिला', 'utf8');
    res.writeHead(404, {
      'Content-Type': 'text/plain; charset=utf-8',
      'Content-Length': body.length,
    });
    res.end(body);
  }
});

srv.listen(8731, '127.0.0.1', () => console.log('lipi http test server on 127.0.0.1:8731'));
