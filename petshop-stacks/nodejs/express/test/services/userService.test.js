const chai = require('chai');
const sinon = require('sinon');

const { expect } = chai;
const { expectRejection } = require('../helpers');

const userRepo = require('../../db/userRepository');
const UserService = require('../../services/UserService');

describe('UserService', () => {
  afterEach(() => sinon.restore());

  describe('createUser', () => {
    it('returns the created user on success', async () => {
      const user = { username: 'alice' };
      sinon.stub(userRepo, 'create').resolves(user);

      const res = await UserService.createUser({ user });

      expect(res).to.deep.equal({ payload: user, code: 200 });
    });

    it('maps errors to status 405 by default', async () => {
      sinon.stub(userRepo, 'create').rejects(new Error('dupe'));

      const rejection = await expectRejection(UserService.createUser({ user: {} }));

      expect(rejection).to.deep.equal({ error: 'dupe', code: 405 });
    });
  });

  describe('createUsersWithListInput', () => {
    it('creates every user and returns the last result', async () => {
      const stub = sinon.stub(userRepo, 'create').callsFake(async (u) => u);

      const res = await UserService.createUsersWithListInput({
        user: [{ username: 'a' }, { username: 'b' }],
      });

      expect(stub.callCount).to.equal(2);
      expect(res).to.deep.equal({ payload: { username: 'b' }, code: 200 });
    });

    it('accepts a single (non-array) user', async () => {
      const stub = sinon.stub(userRepo, 'create').callsFake(async (u) => u);

      const res = await UserService.createUsersWithListInput({ user: { username: 'solo' } });

      expect(stub.calledOnceWithExactly({ username: 'solo' })).to.equal(true);
      expect(res).to.deep.equal({ payload: { username: 'solo' }, code: 200 });
    });

    it('maps errors to status 405 by default', async () => {
      sinon.stub(userRepo, 'create').rejects(new Error('create failed'));

      const rejection = await expectRejection(
        UserService.createUsersWithListInput({ user: [{ username: 'a' }] }),
      );

      expect(rejection).to.deep.equal({ error: 'create failed', code: 405 });
    });
  });

  describe('getUserByName', () => {
    it('returns the user on success', async () => {
      const user = { id: 1, username: 'bob' };
      sinon.stub(userRepo, 'findByUsername').resolves(user);

      const res = await UserService.getUserByName({ username: 'bob' });

      expect(userRepo.findByUsername.calledOnceWithExactly('bob')).to.equal(true);
      expect(res).to.deep.equal({ payload: user, code: 200 });
    });

    it('maps a 404 not-found error preserving e.message and e.status', async () => {
      const err = new Error('No such user: ghost');
      err.status = 404;
      sinon.stub(userRepo, 'findByUsername').rejects(err);

      const rejection = await expectRejection(UserService.getUserByName({ username: 'ghost' }));

      expect(rejection).to.deep.equal({ error: 'No such user: ghost', code: 404 });
    });

    it('defaults code to 404 when error has no status', async () => {
      sinon.stub(userRepo, 'findByUsername').rejects(new Error('gone'));

      const rejection = await expectRejection(UserService.getUserByName({ username: 'x' }));

      expect(rejection.code).to.equal(404);
    });
  });

  describe('updateUser', () => {
    it('returns an empty payload on success', async () => {
      const stub = sinon.stub(userRepo, 'update').resolves();

      const res = await UserService.updateUser({ username: 'bob', user: { email: 'x@y.z' } });

      expect(stub.calledOnceWithExactly('bob', { email: 'x@y.z' })).to.equal(true);
      expect(res).to.deep.equal({ payload: {}, code: 200 });
    });

    it('maps errors with status to rejectResponse', async () => {
      const err = new Error('update failed');
      err.status = 400;
      sinon.stub(userRepo, 'update').rejects(err);

      const rejection = await expectRejection(
        UserService.updateUser({ username: 'alice', user: {} }),
      );

      expect(rejection).to.deep.equal({ error: 'update failed', code: 400 });
    });

    it('defaults to 405 when error has no status', async () => {
      sinon.stub(userRepo, 'update').rejects(new Error('oops'));

      const rejection = await expectRejection(
        UserService.updateUser({ username: 'alice', user: {} }),
      );

      expect(rejection.code).to.equal(405);
    });
  });

  describe('deleteUser', () => {
    it('maps errors to status 400 by default', async () => {
      sinon.stub(userRepo, 'deleteUser').rejects(new Error('bad username'));

      const rejection = await expectRejection(UserService.deleteUser({ username: 'x' }));

      expect(rejection).to.deep.equal({ error: 'bad username', code: 400 });
    });
  });

  describe('loginUser', () => {
    it('resolves with a session string when credentials are valid', async () => {
      sinon.stub(userRepo, 'authenticate').resolves(true);

      const res = await UserService.loginUser({ username: 'bob', password: 'pw' });

      expect(res).to.deep.equal({ payload: 'logged in user session:bob', code: 200 });
    });

    it('rejects with 400 when credentials are invalid', async () => {
      sinon.stub(userRepo, 'authenticate').resolves(false);

      const rejection = await expectRejection(
        UserService.loginUser({ username: 'bob', password: 'wrong' }),
      );

      expect(rejection).to.deep.equal({
        error: 'Invalid username/password supplied',
        code: 400,
      });
    });

    it('maps unexpected exceptions to a 400 error response', async () => {
      sinon.stub(userRepo, 'authenticate').rejects(new Error('auth error'));

      const rejection = await expectRejection(
        UserService.loginUser({ username: 'x', password: 'y' }),
      );

      expect(rejection).to.deep.equal({ error: 'auth error', code: 400 });
    });
  });

  describe('logoutUser', () => {
    it('is a stateless no-op success', async () => {
      const res = await UserService.logoutUser();

      expect(res).to.deep.equal({ payload: {}, code: 200 });
    });
  });
});
