'use strict';
const userRepo = require('../db/userRepository');

async function userRoutes(fastify) {
  fastify.post('/user', async (req, reply) => {
    const user = req.body;
    if (!user || !user.username) {
      return reply.code(400).send({ message: 'Invalid user supplied' });
    }
    const result = await userRepo.create(user);
    return reply.send(result);
  });

  fastify.post('/user/createWithList', async (req, reply) => {
    const users = req.body;
    if (!Array.isArray(users)) {
      return reply.code(400).send({ message: 'Expected array of users' });
    }
    for (const user of users) {
      await userRepo.create(user);
    }
    return reply.send(users);
  });

  // Static routes before parameterized
  fastify.get('/user/login', async (req, reply) => {
    const { username, password } = req.query;
    if (!username || !password) {
      return reply.code(400).send({ message: 'Username and password required' });
    }
    const ok = await userRepo.authenticate(username, password);
    if (!ok) return reply.code(401).send({ message: 'Invalid credentials' });
    return reply.send({ message: 'logged in user session' });
  });

  fastify.get('/user/logout', async (req, reply) => {
    // Stateless no-op
    return reply.code(200).send({});
  });

  fastify.get('/user/:username', async (req, reply) => {
    try {
      const user = await userRepo.findByUsername(req.params.username);
      return reply.send(user);
    } catch (err) {
      if (err.status === 404) return reply.code(404).send({ message: err.message });
      throw err;
    }
  });

  fastify.put('/user/:username', async (req, reply) => {
    const user = req.body;
    if (!user) return reply.code(400).send({ message: 'Invalid user supplied' });
    await userRepo.update(req.params.username, user);
    return reply.code(200).send({});
  });

  fastify.delete('/user/:username', async (req, reply) => {
    await userRepo.deleteUser(req.params.username);
    return reply.code(200).send({});
  });
}

module.exports = userRoutes;
