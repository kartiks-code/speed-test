const Service = require('./Service');
const petRepo = require('../db/petRepository');

const addPet = async ({ pet }) => {
  try {
    const result = await petRepo.add(pet);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const updatePet = async ({ pet }) => {
  try {
    const result = await petRepo.update(pet);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const findPetsByStatus = async ({ status }) => {
  try {
    const result = await petRepo.findByStatus(status || 'available');
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid status value', e.status || 400);
  }
};

const findPetsByTags = async ({ tags }) => {
  try {
    const tagList = Array.isArray(tags) ? tags : (tags ? [tags] : []);
    const result = await petRepo.findByTags(tagList);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid tag value', e.status || 400);
  }
};

const getPetById = async ({ petId }) => {
  try {
    const result = await petRepo.findById(petId);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Pet not found', e.status || 404);
  }
};

const deletePet = async ({ petId }) => {
  try {
    await petRepo.deletePet(petId);
    return Service.successResponse({});
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid pet value', e.status || 400);
  }
};

const updatePetWithForm = async ({ petId, name, status }) => {
  try {
    await petRepo.updateWithForm(petId, name || null, status || null);
    return Service.successResponse({});
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const uploadFile = async ({ petId, additionalMetadata, body }) => {
  try {
    const content = Buffer.isBuffer(body) ? body : Buffer.alloc(0);
    const size = await petRepo.addPhoto(petId, content, 'application/octet-stream', additionalMetadata || null);
    return Service.successResponse({
      code: 200,
      type: 'application/octet-stream',
      message: `File uploaded for pet ${petId}, ${size} bytes${additionalMetadata ? `, metadata: ${additionalMetadata}` : ''}`,
    });
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

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
