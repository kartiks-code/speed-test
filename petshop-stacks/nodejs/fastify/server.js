'use strict';
const fastify = require('fastify');
const petRoutes = require('./routes/pet');
const storeRoutes = require('./routes/store');
const userRoutes = require('./routes/user');

function buildApp(opts = {}) {
  const app = fastify(opts);

  // Handle raw binary uploads for uploadImage
  app.addContentTypeParser('application/octet-stream', { parseAs: 'buffer' }, (req, body, done) => done(null, body));

  // Handle URL-encoded form data for POST /pet/:petId
  app.addContentTypeParser('application/x-www-form-urlencoded', { parseAs: 'string' }, (req, body, done) => {
    const parsed = {};
    if (body) {
      for (const pair of body.split('&')) {
        const [key, val] = pair.split('=').map(decodeURIComponent);
        if (key) parsed[key] = val;
      }
    }
    done(null, parsed);
  });

  app.register(petRoutes, { prefix: '/api/v3' });
  app.register(storeRoutes, { prefix: '/api/v3' });
  app.register(userRoutes, { prefix: '/api/v3' });

  return app;
}

module.exports = { buildApp };
