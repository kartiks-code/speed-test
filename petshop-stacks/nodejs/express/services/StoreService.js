const Service = require('./Service');
const orderRepo = require('../db/orderRepository');
const petRepo = require('../db/petRepository');

const getInventory = async () => {
  try {
    const result = await petRepo.getInventory();
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 500);
  }
};

const placeOrder = async ({ order }) => {
  try {
    const result = await orderRepo.place(order);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const getOrderById = async ({ orderId }) => {
  try {
    const result = await orderRepo.findById(orderId);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Order not found', e.status || 404);
  }
};

const deleteOrder = async ({ orderId }) => {
  try {
    await orderRepo.deleteOrder(orderId);
    return Service.successResponse({});
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid ID supplied', e.status || 400);
  }
};

module.exports = {
  deleteOrder,
  getInventory,
  getOrderById,
  placeOrder,
};
