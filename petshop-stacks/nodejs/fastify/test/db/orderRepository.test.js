const chai = require('chai');
const sinon = require('sinon');

const { expect } = chai;
const { expectRejection } = require('../helpers');

const poolModule = require('../../db/pool');
const orderRepo = require('../../db/orderRepository');

const sql = (q) => q.replace(/\s+/g, ' ').trim();

describe('orderRepository', () => {
  let queryStub;

  beforeEach(() => {
    queryStub = sinon.stub(poolModule.pool, 'query');
  });
  afterEach(() => sinon.restore());

  describe('place', () => {
    it('uses nextval for server-assigned id when none provided', async () => {
      queryStub.onFirstCall().resolves({ rows: [{ id: '77' }] });
      queryStub.onSecondCall().resolves({ rows: [], rowCount: 1 });

      const result = await orderRepo.place({});

      const [nextvalText] = queryStub.firstCall.args;
      expect(nextvalText).to.contain("nextval('order_id_seq')");
      expect(result.id).to.equal(77);
    });

    it('uses provided id without calling nextval', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });

      const result = await orderRepo.place({ id: 100, petId: 5, quantity: 2, status: 'placed', complete: false });

      expect(queryStub.callCount).to.equal(1);
      expect(result.id).to.equal(100);
    });

    it('casts order_status, converts shipDate to a Date, and echoes the order', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });
      const order = {
        id: 100,
        petId: 5,
        quantity: 2,
        shipDate: '2026-01-02T03:04:05.000Z',
        status: 'placed',
        complete: false,
      };

      const result = await orderRepo.place(order);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('INSERT INTO "order"');
      expect(sql(text)).to.contain('cast($5 as order_status)');
      expect(params[0]).to.equal(100);
      expect(params[1]).to.equal(5);
      expect(params[2]).to.equal(2);
      expect(params[3]).to.be.an.instanceof(Date);
      expect(params[3].toISOString()).to.equal('2026-01-02T03:04:05.000Z');
      expect(params[4]).to.equal('placed');
      expect(params[5]).to.equal(false);
      expect(result).to.deep.equal(order);
    });

    it('nulls optional fields when omitted', async () => {
      queryStub.onFirstCall().resolves({ rows: [{ id: '1' }] });
      queryStub.onSecondCall().resolves({ rows: [], rowCount: 1 });

      const result = await orderRepo.place({});

      const params = queryStub.secondCall.args[1];
      expect(result.id).to.be.a('number');
      expect(params[1]).to.equal(null); // petId
      expect(params[2]).to.equal(null); // quantity
      expect(params[3]).to.equal(null); // shipDate
      expect(params[4]).to.equal(null); // status
      expect(params[5]).to.equal(null); // complete
    });
  });

  describe('findById', () => {
    it('maps a row, converting ship_date to an ISO string', async () => {
      const shipDate = new Date('2026-05-06T07:08:09.000Z');
      queryStub.resolves({
        rows: [{
          id: '11',
          pet_id: '3',
          quantity: '4',
          ship_date: shipDate,
          status: 'approved',
          complete: true,
        }],
      });

      const order = await orderRepo.findById(11);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('SELECT "id", pet_id, quantity, ship_date');
      expect(params).to.deep.equal([11]);
      expect(order).to.deep.equal({
        id: 11,
        petId: 3,
        quantity: 4,
        shipDate: '2026-05-06T07:08:09.000Z',
        status: 'approved',
        complete: true,
      });
    });

    it('maps null id, petId, quantity, and complete fields to undefined', async () => {
      queryStub.resolves({
        rows: [{
          id: null,
          pet_id: null,
          quantity: null,
          ship_date: null,
          status: null,
          complete: null,
        }],
      });

      const order = await orderRepo.findById(5);

      expect(order.id).to.equal(undefined);
      expect(order.petId).to.equal(undefined);
      expect(order.quantity).to.equal(undefined);
      expect(order.shipDate).to.equal(undefined);
      expect(order.status).to.equal(undefined);
      expect(order.complete).to.equal(undefined);
    });

    it('throws a 404 when the order is missing', async () => {
      queryStub.resolves({ rows: [] });

      const err = await expectRejection(orderRepo.findById(404));

      expect(err.status).to.equal(404);
      expect(err.message).to.equal('Order not found');
    });
  });

  describe('deleteOrder', () => {
    it('issues a parameterized DELETE', async () => {
      queryStub.resolves({ rowCount: 1 });

      await orderRepo.deleteOrder(8);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('DELETE FROM "order" WHERE "id" = $1');
      expect(params).to.deep.equal([8]);
    });
  });
});
