// Resolves with the rejection reason of a promise, or throws if it resolves.
// Services reject with plain objects (not Error), so a try/catch helper keeps
// the assertion style consistent across the suite.
const expectRejection = async (promise) => promise.then(
  () => { throw new Error('expected promise to be rejected, but it resolved'); },
  (reason) => reason,
);

module.exports = { expectRejection };
