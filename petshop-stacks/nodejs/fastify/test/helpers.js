// Resolves with the rejection reason of a promise, or throws if it resolves.
const expectRejection = async (promise) => promise.then(
  () => { throw new Error('expected promise to be rejected, but it resolved'); },
  (reason) => reason,
);

module.exports = { expectRejection };
