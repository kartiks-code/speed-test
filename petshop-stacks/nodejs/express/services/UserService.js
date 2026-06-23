const Service = require('./Service');
const userRepo = require('../db/userRepository');

const createUser = async ({ user }) => {
  try {
    const result = await userRepo.create(user);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const createUsersWithListInput = async ({ user }) => {
  try {
    const users = Array.isArray(user) ? user : [user];
    // Create all users concurrently instead of sequentially.
    const results = await Promise.all(users.map((u) => userRepo.create(u)));
    return Service.successResponse(results[results.length - 1]);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const getUserByName = async ({ username }) => {
  try {
    const result = await userRepo.findByUsername(username);
    return Service.successResponse(result);
  } catch (e) {
    throw Service.rejectResponse(e.message || 'User not found', e.status || 404);
  }
};

const updateUser = async ({ username, user }) => {
  try {
    await userRepo.update(username, user);
    return Service.successResponse({});
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 405);
  }
};

const deleteUser = async ({ username }) => {
  try {
    await userRepo.deleteUser(username);
    return Service.successResponse({});
  } catch (e) {
    throw Service.rejectResponse(e.message || 'Invalid username supplied', e.status || 400);
  }
};

const loginUser = async ({ username, password }) => {
  try {
    const ok = await userRepo.authenticate(username, password);
    if (!ok) {
      throw Service.rejectResponse('Invalid username/password supplied', 400);
    }
    return Service.successResponse(`logged in user session:${username}`);
  } catch (e) {
    if (e.code !== undefined) throw e; // already a rejectResponse object
    throw Service.rejectResponse(e.message || 'Invalid input', e.status || 400);
  }
};

const logoutUser = async () => Service.successResponse({});

module.exports = {
  createUser,
  createUsersWithListInput,
  deleteUser,
  getUserByName,
  loginUser,
  logoutUser,
  updateUser,
};
