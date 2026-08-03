/* eslint-disable */
// Tiny proxy: JMAP → Stalwart, everything else → Bulwark
// Translates Bulwark's Submission/set → Stalwart's EmailSubmission/set
const http = require('http');

const BULWARK_PORT = parseInt(process.env.BULWARK_PORT || '3001', 10);
const STALWART_HOST = process.env.STALWART_HOST || '127.0.0.1';
const STALWART_PORT = parseInt(process.env.STALWART_PORT || '8080', 10);
const PROXY_PORT = parseInt(process.env.PROXY_PORT || '3000', 10);

// Bulwark uses draft JMAP method names — translate for Stalwart
const METHOD_MAP = {
  'Submission/set': 'EmailSubmission/set',
  'Submission/get': 'EmailSubmission/get',
  'Submission/query': 'EmailSubmission/query',
  'Submission/changes': 'EmailSubmission/changes',
};

function translateJmapBody(buf) {
  try {
    const data = JSON.parse(buf.toString());
    if (data.methodCalls) {
      for (const call of data.methodCalls) {
        if (call[0] && METHOD_MAP[call[0]]) {
          call[0] = METHOD_MAP[call[0]];
        }
      }
    }
    return JSON.stringify(data);
  } catch { return buf.toString(); }
}

function proxyTo(targetHost, targetPort, req, res) {
  const isJmapPost = req.method === 'POST' && (req.url.startsWith('/.well-known/jmap') || req.url.startsWith('/jmap/'));
  const body = ['POST', 'PUT', 'PATCH'].includes(req.method) ? [] : null;
  const headers = {};
  for (const [k, v] of Object.entries(req.headers)) {
    if (v !== undefined && k !== 'host') headers[k] = v;
  }
  headers['host'] = targetHost + ':' + targetPort;

  const opts = { hostname: targetHost, port: targetPort, path: req.url, method: req.method, headers };
  const proxy = http.request(opts, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });
  proxy.on('error', (e) => { console.error('[proxy]', e.message); res.statusCode = 502; res.end('Bad Gateway'); });

  if (body) {
    req.on('data', (chunk) => body.push(chunk));
    req.on('end', () => {
      let raw = Buffer.concat(body);
      if (isJmapPost) raw = Buffer.from(translateJmapBody(raw));
      headers['content-length'] = String(raw.length);
      proxy.write(raw);
      proxy.end();
    });
  } else {
    proxy.end();
  }
}

const server = http.createServer((req, res) => {
  const url = req.url || '';
  if (url.startsWith('/.well-known/jmap') || url.startsWith('/jmap/')) {
    proxyTo(STALWART_HOST, STALWART_PORT, req, res);
  } else {
    proxyTo('127.0.0.1', BULWARK_PORT, req, res);
  }
});

server.listen(PROXY_PORT, '0.0.0.0', () => {
  console.log(`Proxy on :${PROXY_PORT} → JMAP:${STALWART_PORT} | Web:${BULWARK_PORT}`);
});
