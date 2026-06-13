'use strict';
const petRepo = require('../db/petRepository');

async function petRoutes(fastify) {
  fastify.post('/pet', async (req, reply) => {
    const pet = req.body;
    if (!pet || !pet.name || !pet.photoUrls || pet.photoUrls.length === 0) {
      return reply.code(405).send({ message: 'Invalid input' });
    }
    const result = await petRepo.add(pet);
    return reply.send(result);
  });

  fastify.put('/pet', async (req, reply) => {
    const pet = req.body;
    if (!pet || !pet.id) {
      return reply.code(400).send({ message: 'Invalid ID supplied' });
    }
    try {
      const result = await petRepo.update(pet);
      return reply.send(result);
    } catch (err) {
      if (err.status === 404) return reply.code(404).send({ message: err.message });
      if (err.status === 400) return reply.code(400).send({ message: err.message });
      throw err;
    }
  });

  // Static routes must come before parameterized ones
  fastify.get('/pet/findByStatus', async (req, reply) => {
    const status = req.query.status || 'available';
    const pets = await petRepo.findByStatus(status);
    return reply.send(pets);
  });

  fastify.get('/pet/findByTags', async (req, reply) => {
    const raw = req.query.tags;
    const tags = Array.isArray(raw) ? raw : (raw ? [raw] : []);
    const pets = await petRepo.findByTags(tags);
    return reply.send(pets);
  });

  fastify.get('/pet/:petId', async (req, reply) => {
    const id = parseInt(req.params.petId, 10);
    if (isNaN(id)) return reply.code(400).send({ message: 'Invalid pet ID' });
    try {
      const pet = await petRepo.findById(id);
      return reply.send(pet);
    } catch (err) {
      if (err.status === 404) return reply.code(404).send({ message: err.message });
      throw err;
    }
  });

  fastify.post('/pet/:petId', async (req, reply) => {
    const id = parseInt(req.params.petId, 10);
    if (isNaN(id)) return reply.code(400).send({ message: 'Invalid pet ID' });
    const { name, status } = req.body || {};
    await petRepo.updateWithForm(id, name || null, status || null);
    return reply.code(200).send({});
  });

  fastify.delete('/pet/:petId', async (req, reply) => {
    const id = parseInt(req.params.petId, 10);
    if (isNaN(id)) return reply.code(400).send({ message: 'Invalid pet ID' });
    await petRepo.deletePet(id);
    return reply.code(200).send({});
  });

  fastify.post('/pet/:petId/uploadImage', async (req, reply) => {
    const id = parseInt(req.params.petId, 10);
    if (isNaN(id)) return reply.code(400).send({ message: 'Invalid pet ID' });
    const body = req.body;
    const bytes = Buffer.isBuffer(body) ? body : Buffer.from(body || '');
    try {
      const byteCount = await petRepo.addPhoto(id, bytes, 'application/octet-stream', null);
      return reply.send({ message: `File uploaded, ${byteCount} bytes stored` });
    } catch (err) {
      if (err.status === 404) return reply.code(404).send({ message: err.message });
      throw err;
    }
  });
}

module.exports = petRoutes;
