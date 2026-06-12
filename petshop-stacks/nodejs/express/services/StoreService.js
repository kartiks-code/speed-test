const Service = require('./Service');
const orderRepo = require('../db/orderRepository');
const petRepo = require('../db/petRepository');

const getInventory = () => new Promise(
  async (resolve, reject) => {
    try {
      const result = await petRepo.getInventory();
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 500));
    }
  },
);

const placeOrder = ({ order }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await orderRepo.place(order);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const getOrderById = ({ orderId }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await orderRepo.findById(orderId);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Order not found', e.status || 404));
    }
  },
);

const deleteOrder = ({ orderId }) => new Promise(
  async (resolve, reject) => {
    try {
      await orderRepo.deleteOrder(orderId);
      resolve(Service.successResponse({}));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid ID supplied', e.status || 400));
    }
  },
);

module.exports = {
  deleteOrder,
  getInventory,
  getOrderById,
  placeOrder,
};
