const chai = require('chai');
const sinon = require('sinon');

const { expect } = chai;
const { expectRejection } = require('../helpers');

const poolModule = require('../../db/pool');
const userRepo = require('../../db/userRepository');

const sql = (q) => q.replace(/\s+/g, ' ').trim();

describe('userRepository', () => {
  let queryStub;

  beforeEach(() => {
    queryStub = sinon.stub(poolModule.pool, 'query');
  });
  afterEach(() => sinon.restore());

  describe('create', () => {
    it('inserts the user with null-coalesced optional fields', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });
      const user = {
        id: 1,
        username: 'alice',
        firstName: 'Alice',
        email: 'a@example.com',
        userStatus: 1,
      };

      const result = await userRepo.create(user);

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('INSERT INTO "user"');
      expect(params[0]).to.equal(1);
      expect(params[1]).to.equal('alice');
      expect(params[2]).to.equal('Alice');
      expect(params[3]).to.equal(null); // lastName
      expect(params[4]).to.equal('a@example.com');
      expect(params[5]).to.equal(null); // password
      expect(params[6]).to.equal(null); // phone
      expect(params[7]).to.equal(1);    // userStatus
      expect(result).to.deep.equal(user);
    });

    it('passes null for absent id and userStatus', async () => {
      queryStub.resolves({ rows: [], rowCount: 1 });

      await userRepo.create({ username: 'noId' });

      const params = queryStub.firstCall.args[1];
      expect(params[0]).to.equal(null); // id absent → null
      expect(params[7]).to.equal(null); // userStatus absent → null
    });
  });

  describe('findByUsername', () => {
    it('maps a row into the API shape', async () => {
      queryStub.resolves({
        rows: [{
          id: '2',
          username: 'bob',
          first_name: 'Bob',
          last_name: 'Jones',
          email: 'b@example.com',
          password: 'pw',
          phone: '555',
          user_status: '0',
        }],
      });

      const user = await userRepo.findByUsername('bob');

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('SELECT "id"');
      expect(sql(text)).to.contain('WHERE username = $1');
      expect(params).to.deep.equal(['bob']);
      expect(user).to.deep.equal({
        id: 2,
        username: 'bob',
        firstName: 'Bob',
        lastName: 'Jones',
        email: 'b@example.com',
        password: 'pw',
        phone: '555',
        userStatus: 0,
      });
    });

    it('maps a null id and null user_status to undefined', async () => {
      queryStub.resolves({
        rows: [{
          id: null,
          username: 'nulluser',
          first_name: null,
          last_name: null,
          email: null,
          password: null,
          phone: null,
          user_status: null,
        }],
      });

      const user = await userRepo.findByUsername('nulluser');

      expect(user.id).to.equal(undefined);
      expect(user.userStatus).to.equal(undefined);
    });

    it('throws a 404 when the user does not exist', async () => {
      queryStub.resolves({ rows: [] });

      const err = await expectRejection(userRepo.findByUsername('ghost'));

      expect(err.status).to.equal(404);
      expect(err.message).to.equal('User not found');
    });
  });

  describe('update', () => {
    it('binds the username as the final WHERE param', async () => {
      queryStub.resolves({ rowCount: 1 });

      await userRepo.update('bob', { email: 'new@example.com' });

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('UPDATE "user" SET');
      expect(sql(text)).to.contain('WHERE username = $8');
      expect(params[3]).to.equal('new@example.com'); // email
      expect(params[7]).to.equal('bob');
    });

    it('binds all provided fields correctly', async () => {
      queryStub.resolves({ rowCount: 1 });

      await userRepo.update('alice', {
        id: 5,
        firstName: 'Alice',
        lastName: 'Smith',
        email: 'alice@example.com',
        password: 'secret',
        phone: '123-456',
        userStatus: 2,
      });

      const params = queryStub.firstCall.args[1];
      expect(params[0]).to.equal(5);                    // id
      expect(params[1]).to.equal('Alice');               // firstName
      expect(params[2]).to.equal('Smith');               // lastName
      expect(params[3]).to.equal('alice@example.com');   // email
      expect(params[4]).to.equal('secret');              // password
      expect(params[5]).to.equal('123-456');             // phone
      expect(params[6]).to.equal(2);                     // userStatus
      expect(params[7]).to.equal('alice');               // username (WHERE)
    });

    it('passes null for absent optional fields', async () => {
      queryStub.resolves({ rowCount: 1 });

      await userRepo.update('alice', { username: 'alice' });

      const params = queryStub.firstCall.args[1];
      expect(params[0]).to.equal(null); // id → null
      expect(params[1]).to.equal(null); // firstName → null
      expect(params[2]).to.equal(null); // lastName → null
      expect(params[4]).to.equal(null); // password → null
      expect(params[5]).to.equal(null); // phone → null
      expect(params[6]).to.equal(null); // userStatus → null
    });
  });

  describe('deleteUser', () => {
    it('issues a parameterized DELETE', async () => {
      queryStub.resolves({ rowCount: 1 });

      await userRepo.deleteUser('bob');

      const [text, params] = queryStub.firstCall.args;
      expect(sql(text)).to.contain('DELETE FROM "user" WHERE username = $1');
      expect(params).to.deep.equal(['bob']);
    });
  });

  describe('authenticate', () => {
    it('returns true when a matching credential row exists', async () => {
      queryStub.resolves({ rows: [{ '?column?': 1 }] });

      const ok = await userRepo.authenticate('bob', 'pw');

      const [text, params] = queryStub.firstCall.args;
      expect(text).to.contain('username = $1 AND "password" = $2');
      expect(params).to.deep.equal(['bob', 'pw']);
      expect(ok).to.equal(true);
    });

    it('returns false when no row matches', async () => {
      queryStub.resolves({ rows: [] });

      const ok = await userRepo.authenticate('bob', 'wrong');

      expect(ok).to.equal(false);
    });
  });
});
