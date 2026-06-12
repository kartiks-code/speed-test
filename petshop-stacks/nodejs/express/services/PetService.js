const Service = require('./Service');
const petRepo = require('../db/petRepository');

const addPet = ({ pet }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await petRepo.add(pet);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const updatePet = ({ pet }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await petRepo.update(pet);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const findPetsByStatus = ({ status }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await petRepo.findByStatus(status || 'available');
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid status value', e.status || 400));
    }
  },
);

const findPetsByTags = ({ tags }) => new Promise(
  async (resolve, reject) => {
    try {
      const tagList = Array.isArray(tags) ? tags : (tags ? [tags] : []);
      const result = await petRepo.findByTags(tagList);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid tag value', e.status || 400));
    }
  },
);

const getPetById = ({ petId }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await petRepo.findById(petId);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Pet not found', e.status || 404));
    }
  },
);

const deletePet = ({ petId }) => new Promise(
  async (resolve, reject) => {
    try {
      await petRepo.deletePet(petId);
      resolve(Service.successResponse({}));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid pet value', e.status || 400));
    }
  },
);

const updatePetWithForm = ({ petId, name, status }) => new Promise(
  async (resolve, reject) => {
    try {
      await petRepo.updateWithForm(petId, name || null, status || null);
      resolve(Service.successResponse({}));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const uploadFile = ({ petId, additionalMetadata, body }) => new Promise(
  async (resolve, reject) => {
    try {
      const content = Buffer.isBuffer(body) ? body : Buffer.alloc(0);
      const size = await petRepo.addPhoto(petId, content, 'application/octet-stream', additionalMetadata || null);
      resolve(Service.successResponse({
        code: 200,
        type: 'application/octet-stream',
        message: `File uploaded for pet ${petId}, ${size} bytes${additionalMetadata ? `, metadata: ${additionalMetadata}` : ''}`,
      }));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

module.exports = {
  addPet,
  deletePet,
  findPetsByStatus,
  findPetsByTags,
  getPetById,
  updatePet,
  updatePetWithForm,
  uploadFile,
};
