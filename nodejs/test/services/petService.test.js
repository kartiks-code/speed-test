const chai = require('chai');
const chaiAsPromised = require('chai-as-promised');
const sinon = require('sinon');

chai.use(chaiAsPromised);
const { expect } = chai;

const petRepo = require('../../db/petRepository');
const PetService = require('../../services/PetService');

describe('PetService', () => {
  afterEach(() => sinon.restore());

  describe('addPet', () => {
    it('wraps the repository result in a success response', async () => {
      const created = { id: 1, name: 'Rex', photoUrls: [] };
      sinon.stub(petRepo, 'add').resolves(created);

      const res = await PetService.addPet({ pet: { name: 'Rex', photoUrls: [] } });

      expect(res).to.deep.equal({ payload: created, code: 200 });
    });

    it('maps a repository error with .status to rejectResponse', async () => {
      const err = new Error('boom');
      err.status = 400;
      sinon.stub(petRepo, 'add').rejects(err);

      const rejection = await PetService.addPet({ pet: {} }).then(
        () => { throw new Error('expected rejection'); },
        (e) => e,
      );

      expect(rejection).to.deep.equal({ error: 'boom', code: 400 });
    });

    it('defaults to 405 when the error has no status', async () => {
      sinon.stub(petRepo, 'add').rejects(new Error('nope'));

      const rejection = await PetService.addPet({ pet: {} }).then(
        () => { throw new Error('expected rejection'); },
        (e) => e,
      );

      expect(rejection).to.deep.equal({ error: 'nope', code: 405 });
    });
  });

  describe('getPetById', () => {
    it('returns the pet on success', async () => {
      const pet = { id: 7, name: 'Milo', photoUrls: [] };
      sinon.stub(petRepo, 'findById').resolves(pet);

      const res = await PetService.getPetById({ petId: 7 });

      expect(petRepo.findById.calledOnceWithExactly(7)).to.equal(true);
      expect(res).to.deep.equal({ payload: pet, code: 200 });
    });

    it('maps a 404 not-found error', async () => {
      const err = new Error('Pet not found');
      err.status = 404;
      sinon.stub(petRepo, 'findById').rejects(err);

      const rejection = await PetService.getPetById({ petId: 99 }).then(
        () => { throw new Error('expected rejection'); },
        (e) => e,
      );

      expect(rejection).to.deep.equal({ error: 'Pet not found', code: 404 });
    });
  });

  describe('findPetsByStatus', () => {
    it("defaults to 'available' when status is omitted", async () => {
      const stub = sinon.stub(petRepo, 'findByStatus').resolves([]);

      await PetService.findPetsByStatus({});

      expect(stub.calledOnceWithExactly('available')).to.equal(true);
    });

    it('passes through an explicit status', async () => {
      const stub = sinon.stub(petRepo, 'findByStatus').resolves([{ id: 1 }]);

      const res = await PetService.findPetsByStatus({ status: 'sold' });

      expect(stub.calledOnceWithExactly('sold')).to.equal(true);
      expect(res).to.deep.equal({ payload: [{ id: 1 }], code: 200 });
    });

    it('maps repository errors to status 400 by default', async () => {
      sinon.stub(petRepo, 'findByStatus').rejects(new Error('bad status'));

      const rejection = await PetService.findPetsByStatus({ status: 'x' }).then(
        () => { throw new Error('expected rejection'); },
        (e) => e,
      );

      expect(rejection).to.deep.equal({ error: 'bad status', code: 400 });
    });
  });

  describe('findPetsByTags', () => {
    it('normalizes a single string tag into an array', async () => {
      const stub = sinon.stub(petRepo, 'findByTags').resolves([]);

      await PetService.findPetsByTags({ tags: 'cute' });

      expect(stub.calledOnceWithExactly(['cute'])).to.equal(true);
    });

    it('passes an array of tags through unchanged', async () => {
      const stub = sinon.stub(petRepo, 'findByTags').resolves([]);

      await PetService.findPetsByTags({ tags: ['a', 'b'] });

      expect(stub.calledOnceWithExactly(['a', 'b'])).to.equal(true);
    });

    it('uses an empty array when no tags are provided', async () => {
      const stub = sinon.stub(petRepo, 'findByTags').resolves([]);

      await PetService.findPetsByTags({});

      expect(stub.calledOnceWithExactly([])).to.equal(true);
    });
  });

  describe('deletePet', () => {
    it('returns an empty payload on success', async () => {
      const stub = sinon.stub(petRepo, 'deletePet').resolves();

      const res = await PetService.deletePet({ petId: 5 });

      expect(stub.calledOnceWithExactly(5)).to.equal(true);
      expect(res).to.deep.equal({ payload: {}, code: 200 });
    });
  });

  describe('updatePet', () => {
    it('maps a 404 from the repository', async () => {
      const err = new Error('Pet not found');
      err.status = 404;
      sinon.stub(petRepo, 'update').rejects(err);

      const rejection = await PetService.updatePet({ pet: { id: 1 } }).then(
        () => { throw new Error('expected rejection'); },
        (e) => e,
      );

      expect(rejection).to.deep.equal({ error: 'Pet not found', code: 404 });
    });
  });

  describe('uploadFile', () => {
    it('persists the uploaded bytes and reports the byte count', async () => {
      const content = Buffer.from('image-bytes');
      const addPhoto = sinon.stub(petRepo, 'addPhoto').resolves(content.length);

      const res = await PetService.uploadFile({ petId: 3, additionalMetadata: 'meta', body: content });

      expect(res.code).to.equal(200);
      expect(res.payload.code).to.equal(200);
      expect(res.payload.message).to.contain('pet 3');
      expect(res.payload.message).to.contain(`${content.length} bytes`);
      expect(res.payload.message).to.contain('meta');
      expect(addPhoto.calledOnceWith(3, content, 'application/octet-stream', 'meta')).to.equal(true);
    });

    it('maps a repository error with .status to rejectResponse', async () => {
      const err = new Error('Pet not found');
      err.status = 404;
      sinon.stub(petRepo, 'addPhoto').rejects(err);

      const rejection = await PetService.uploadFile({ petId: 9, body: Buffer.alloc(0) }).then(
        () => { throw new Error('expected rejection'); },
        (e) => e,
      );

      expect(rejection).to.deep.equal({ error: 'Pet not found', code: 404 });
    });
  });
});
