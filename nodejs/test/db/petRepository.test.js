const chai = require('chai');
const sinon = require('sinon');

const { expect } = chai;
const { expectRejection } = require('../helpers');

// The repository captures `query` from ./pool, and that closure calls
// `pool.query(...)` at call time — so stubbing the Pool instance's `query`
// method intercepts every DB call without a live Postgres connection.
const poolModule = require('../../db/pool');
const petRepo = require('../../db/petRepository');

// Normalize whitespace so SQL assertions are resilient to indentation.
const sql = (q) => q.replace(/\s+/g, ' ').trim();

describe('petRepository', () => {
  let queryStub;

  beforeEach(() => {
    queryStub = sinon.stub(poolModule.pool, 'query');
  });
  afterEach(() => sinon.restore());

  describe('add', () => {
    it('serializes JSON columns and casts enums, returning the pet with its id', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });
      const pet = {
        id: 42,
        name: 'Rex',
        category: { id: 1, name: 'Dogs' },
        photoUrls: ['http://img/1.png'],
        tags: [{ id: 5, name: 'cute' }],
        status: 'available',
      };

      const result = await petRepo.add(pet);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('INSERT INTO pet');
      expect(sql(text)).to.contain('cast($4 as json)');
      expect(sql(text)).to.contain('cast($6 as pet_status)');
      expect(params).to.deep.equal([
        42,
        'Rex',
        JSON.stringify({ id: 1, name: 'Dogs' }),
        JSON.stringify(['http://img/1.png']),
        JSON.stringify([{ id: 5, name: 'cute' }]),
        'available',
      ]);
      expect(result).to.deep.equal(pet);
    });

    it('generates a server-side id when none is supplied', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });

      const first = await petRepo.add({ name: 'A', photoUrls: [] });
      const second = await petRepo.add({ name: 'B', photoUrls: [] });

      expect(first.id).to.be.a('number');
      expect(second.id).to.be.a('number');
      // counter is monotonic
      expect(second.id).to.be.greaterThan(first.id);
      expect(queryStub.firstCall.args[1][0]).to.equal(first.id);
    });

    it('passes null for absent category/tags/status', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });

      await petRepo.add({ id: 1, name: 'Bare', photoUrls: [] });

      const params = queryStub.firstCall.args[1];
      expect(params[2]).to.equal(null); // category
      expect(params[4]).to.equal(null); // tags
      expect(params[5]).to.equal(null); // status
    });
  });

  describe('update', () => {
    it('throws a 400 error when the pet id is missing', async () => {
      const err = await expectRejection(petRepo.update({ name: 'NoId', photoUrls: [] }));

      expect(err).to.be.an.instanceof(Error);
      expect(err.status).to.equal(400);
      expect(queryStub.called).to.equal(false);
    });

    it('throws a 404 error when no row is updated', async () => {
      queryStub.resolves({ rowCount: 0 });

      const err = await expectRejection(petRepo.update({ id: 7, name: 'Gone', photoUrls: [] }));

      expect(err.status).to.equal(404);
    });

    it('returns the pet when the update affects a row', async () => {
      queryStub.resolves({ rowCount: 1 });
      const pet = { id: 7, name: 'Up', photoUrls: [], status: 'sold' };

      const result = await petRepo.update(pet);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('UPDATE pet SET');
      expect(params[5]).to.equal(7); // id is the last bound param (WHERE clause)
      expect(result).to.deep.equal(pet);
    });
  });

  describe('findById', () => {
    it('maps a row, parsing the category JSON string and json columns', async () => {
      queryStub.resolves({
        rows: [{
          id: '3',
          name: 'Milo',
          category: '{"id":1,"name":"Dogs"}',
          photo_urls: ['http://img/a.png'],
          tags: [{ id: 2, name: 'tag' }],
          status: 'available',
        }],
      });

      const pet = await petRepo.findById(3);

      expect(queryStub.firstCall.args[1]).to.deep.equal([3]);
      expect(pet).to.deep.equal({
        id: 3,
        name: 'Milo',
        category: { id: 1, name: 'Dogs' },
        photoUrls: ['http://img/a.png'],
        tags: [{ id: 2, name: 'tag' }],
        status: 'available',
      });
    });

    it('defaults photoUrls to an empty array and omits absent fields', async () => {
      queryStub.resolves({
        rows: [{
          id: '4', name: 'Bare', category: null, photo_urls: null, tags: null, status: null,
        }],
      });

      const pet = await petRepo.findById(4);

      expect(pet.photoUrls).to.deep.equal([]);
      expect(pet.category).to.equal(undefined);
      expect(pet.tags).to.equal(undefined);
      expect(pet.status).to.equal(undefined);
    });

    it('throws a 404 when no pet is found', async () => {
      queryStub.resolves({ rows: [] });

      const err = await expectRejection(petRepo.findById(99));

      expect(err.status).to.equal(404);
      expect(err.message).to.equal('Pet not found');
    });
  });

  describe('findByStatus', () => {
    it('casts the status param and maps each row', async () => {
      queryStub.resolves({
        rows: [
          { id: '1', name: 'A', category: null, photo_urls: [], tags: null, status: 'sold' },
          { id: '2', name: 'B', category: null, photo_urls: [], tags: null, status: 'sold' },
        ],
      });

      const pets = await petRepo.findByStatus('sold');

      expect(sql(queryStub.firstCall.args[0])).to.contain('cast($1 as pet_status)');
      expect(queryStub.firstCall.args[1]).to.deep.equal(['sold']);
      expect(pets.map((p) => p.id)).to.deep.equal([1, 2]);
    });
  });

  describe('findByTags', () => {
    it('returns [] without hitting the DB when no tags are given', async () => {
      const result = await petRepo.findByTags([]);

      expect(result).to.deep.equal([]);
      expect(queryStub.called).to.equal(false);
    });

    it('builds one OR-ed jsonb condition and param per tag', async () => {
      queryStub.resolves({ rows: [] });

      await petRepo.findByTags(['cute', 'small']);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('tags::jsonb @> cast($1 as jsonb) OR tags::jsonb @> cast($2 as jsonb)');
      expect(params).to.deep.equal([
        JSON.stringify([{ name: 'cute' }]),
        JSON.stringify([{ name: 'small' }]),
      ]);
    });
  });

  describe('updateWithForm', () => {
    it('does nothing when both name and status are null', async () => {
      await petRepo.updateWithForm(1, null, null);

      expect(queryStub.called).to.equal(false);
    });

    it('builds a dynamic SET clause for the provided fields only', async () => {
      queryStub.resolves({ rowCount: 1 });

      await petRepo.updateWithForm(9, 'NewName', 'pending');

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('"name" = $1');
      expect(sql(text)).to.contain('status = cast($2 as pet_status)');
      expect(sql(text)).to.contain('WHERE "id" = $3');
      expect(params).to.deep.equal(['NewName', 'pending', 9]);
    });

    it('only sets status when name is omitted', async () => {
      queryStub.resolves({ rowCount: 1 });

      await petRepo.updateWithForm(9, null, 'sold');

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('status = cast($1 as pet_status)');
      expect(sql(text)).to.contain('WHERE "id" = $2');
      expect(params).to.deep.equal(['sold', 9]);
    });
  });

  describe('getInventory', () => {
    it('aggregates non-null statuses into a count map', async () => {
      queryStub.resolves({
        rows: [
          { status: 'available', cnt: 3 },
          { status: 'sold', cnt: 1 },
          { status: null, cnt: 9 },
        ],
      });

      const inventory = await petRepo.getInventory();

      expect(inventory).to.deep.equal({ available: 3, sold: 1 });
    });
  });
});
