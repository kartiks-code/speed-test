const Service = require('./Service');
const userRepo = require('../db/userRepository');

const createUser = ({ user }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await userRepo.create(user);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const createUsersWithListInput = ({ user }) => new Promise(
  async (resolve, reject) => {
    try {
      const users = Array.isArray(user) ? user : [user];
      let last;
      for (const u of users) {
        last = await userRepo.create(u);
      }
      resolve(Service.successResponse(last));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const getUserByName = ({ username }) => new Promise(
  async (resolve, reject) => {
    try {
      const result = await userRepo.findByUsername(username);
      resolve(Service.successResponse(result));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'User not found', e.status || 404));
    }
  },
);

const updateUser = ({ username, user }) => new Promise(
  async (resolve, reject) => {
    try {
      await userRepo.update(username, user);
      resolve(Service.successResponse({}));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

const deleteUser = ({ username }) => new Promise(
  async (resolve, reject) => {
    try {
      await userRepo.deleteUser(username);
      resolve(Service.successResponse({}));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid username supplied', e.status || 400));
    }
  },
);

const loginUser = ({ username, password }) => new Promise(
  async (resolve, reject) => {
    try {
      const ok = await userRepo.authenticate(username, password);
      if (!ok) {
        reject(Service.rejectResponse('Invalid username/password supplied', 400));
        return;
      }
      resolve(Service.successResponse(`logged in user session:${username}`));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 400));
    }
  },
);

const logoutUser = () => new Promise(
  async (resolve, reject) => {
    try {
      resolve(Service.successResponse({}));
    } catch (e) {
      reject(Service.rejectResponse(e.message || 'Invalid input', e.status || 405));
    }
  },
);

module.exports = {
  createUser,
  createUsersWithListInput,
  deleteUser,
  getUserByName,
  loginUser,
  logoutUser,
  updateUser,
};
