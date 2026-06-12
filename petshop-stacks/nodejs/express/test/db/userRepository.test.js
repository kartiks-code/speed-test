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
      expect(params[7]).to.equal(1); // userStatus
      expect(result).to.deep.equal(user);
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

      expect(queryStub.firstCall.args[1]).to.deep.equal(['bob']);
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

      expect(queryStub.firstCall.args[1]).to.deep.equal(['bob', 'pw']);
      expect(ok).to.equal(true);
    });

    it('returns false when no row matches', async () => {
      queryStub.resolves({ rows: [] });

      const ok = await userRepo.authenticate('bob', 'wrong');

      expect(ok).to.equal(false);
    });
  });
});
