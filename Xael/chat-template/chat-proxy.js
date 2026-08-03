// chat-proxy.js — Tiny proxy for Valis Chat UI
// Forwards /v1/* to Xael on :4005, everything else to Chat UI on :4024
const http = require('http');

const XAEL = { host: '127.0.0.1', port: 4005 };
const CHAT = { host: '127.0.0.1', port: 4024 };
const PORT = 4024;

http.createServer((req, res) => {
  const target = req.url.startsWith('/v1/') ? XAEL : CHAT;
  const opts = { hostname: target.host, port: target.port, path: req.url, method: req.method, headers: req.headers };
  const proxy = http.request(opts, (pr) => { res.writeHead(pr.statusCode, pr.headers); pr.pipe(res); });
  req.pipe(proxy);
  proxy.on('error', () => { res.writeHead(502); res.end('Bad Gateway'); });
}).listen(PORT, () => console.log(`Chat proxy on :${PORT} → Xael :4005 + Chat :4024`));
