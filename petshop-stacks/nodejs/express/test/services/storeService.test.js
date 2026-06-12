const chai = require('chai');
const sinon = require('sinon');

const { expect } = chai;
const { expectRejection } = require('../helpers');

const orderRepo = require('../../db/orderRepository');
const petRepo = require('../../db/petRepository');
const StoreService = require('../../services/StoreService');

describe('StoreService', () => {
  afterEach(() => sinon.restore());

  describe('getInventory', () => {
    it('returns the inventory map from the pet repository', async () => {
      const inventory = { available: 3, sold: 1 };
      sinon.stub(petRepo, 'getInventory').resolves(inventory);

      const res = await StoreService.getInventory();

      expect(res).to.deep.equal({ payload: inventory, code: 200 });
    });

    it('maps errors to status 500 by default', async () => {
      sinon.stub(petRepo, 'getInventory').rejects(new Error('db down'));

      const rejection = await expectRejection(StoreService.getInventory());

      expect(rejection).to.deep.equal({ error: 'db down', code: 500 });
    });
  });

  describe('placeOrder', () => {
    it('returns the placed order on success', async () => {
      const order = { id: 10, petId: 1, quantity: 2 };
      sinon.stub(orderRepo, 'place').resolves(order);

      const res = await StoreService.placeOrder({ order });

      expect(orderRepo.place.calledOnceWithExactly(order)).to.equal(true);
      expect(res).to.deep.equal({ payload: order, code: 200 });
    });

    it('maps errors to status 405 by default', async () => {
      sinon.stub(orderRepo, 'place').rejects(new Error('bad order'));

      const rejection = await expectRejection(StoreService.placeOrder({ order: {} }));

      expect(rejection).to.deep.equal({ error: 'bad order', code: 405 });
    });
  });

  describe('getOrderById', () => {
    it('returns the order on success', async () => {
      const order = { id: 4, petId: 2, quantity: 1 };
      sinon.stub(orderRepo, 'findById').resolves(order);

      const res = await StoreService.getOrderById({ orderId: 4 });

      expect(orderRepo.findById.calledOnceWithExactly(4)).to.equal(true);
      expect(res).to.deep.equal({ payload: order, code: 200 });
    });

    it('maps a 404 not-found error', async () => {
      const err = new Error('Order not found');
      err.status = 404;
      sinon.stub(orderRepo, 'findById').rejects(err);

      const rejection = await expectRejection(StoreService.getOrderById({ orderId: 999 }));

      expect(rejection).to.deep.equal({ error: 'Order not found', code: 404 });
    });
  });

  describe('deleteOrder', () => {
    it('returns an empty payload on success', async () => {
      const stub = sinon.stub(orderRepo, 'deleteOrder').resolves();

      const res = await StoreService.deleteOrder({ orderId: 8 });

      expect(stub.calledOnceWithExactly(8)).to.equal(true);
      expect(res).to.deep.equal({ payload: {}, code: 200 });
    });

    it('maps errors to status 400 by default', async () => {
      sinon.stub(orderRepo, 'deleteOrder').rejects(new Error('bad id'));

      const rejection = await expectRejection(StoreService.deleteOrder({ orderId: 'x' }));

      expect(rejection).to.deep.equal({ error: 'bad id', code: 400 });
    });
  });
});
