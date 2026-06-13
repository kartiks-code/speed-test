'use strict';
const orderRepo = require('../db/orderRepository');
const petRepo = require('../db/petRepository');

async function storeRoutes(fastify) {
  fastify.get('/store/inventory', async (req, reply) => {
    const inventory = await petRepo.getInventory();
    return reply.send(inventory);
  });

  fastify.post('/store/order', async (req, reply) => {
    const order = req.body;
    if (!order) {
      return reply.code(400).send({ message: 'Invalid order supplied' });
    }
    const result = await orderRepo.place(order);
    return reply.send(result);
  });

  fastify.get('/store/order/:orderId', async (req, reply) => {
    const id = parseInt(req.params.orderId, 10);
    if (isNaN(id)) return reply.code(400).send({ message: 'Invalid order ID' });
    try {
      const order = await orderRepo.findById(id);
      return reply.send(order);
    } catch (err) {
      if (err.status === 404) return reply.code(404).send({ message: err.message });
      throw err;
    }
  });

  fastify.delete('/store/order/:orderId', async (req, reply) => {
    const id = parseInt(req.params.orderId, 10);
    if (isNaN(id)) return reply.code(400).send({ message: 'Invalid order ID' });
    await orderRepo.deleteOrder(id);
    return reply.code(200).send({});
  });
}

module.exports = storeRoutes;
