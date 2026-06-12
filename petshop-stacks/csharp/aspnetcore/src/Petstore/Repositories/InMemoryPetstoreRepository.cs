using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Newtonsoft.Json;
using Petstore.Models;

namespace Petstore.Repositories
{
    public class InMemoryPetstoreRepository : IPetstoreRepository
    {
        private readonly Dictionary<long, Pet>    _pets    = new();
        private readonly Dictionary<long, Order>  _orders  = new();
        private readonly Dictionary<string, User> _users   = new();
        private long _nextPetId    = 1;
        private long _nextOrderId  = 1;
        private long _nextUserId   = 1;

        // ══════════════════════ PET OPERATIONS ════════════════════════

        public Pet AddPet(Pet pet)
        {
            if (pet.Id == 0)
                pet.Id = _nextPetId++;
            else if (pet.Id >= _nextPetId)
                _nextPetId = pet.Id + 1;

            _pets[pet.Id] = Clone(pet);
            return Clone(_pets[pet.Id]);
        }

        public bool DeletePet(long petId)
        {
            return _pets.Remove(petId);
        }

        public List<Pet> FindPetsByStatus(string status)
        {
            if (string.IsNullOrEmpty(status))
                return _pets.Values.Select(Clone).ToList();

            return _pets.Values
                .Where(p => PetStatusToString(p.Status) == status)
                .Select(Clone)
                .ToList();
        }

        public List<Pet> FindPetsByTags(List<string> tags)
        {
            if (tags == null || tags.Count == 0)
                return _pets.Values.Select(Clone).ToList();

            return _pets.Values
                .Where(p => p.Tags != null && p.Tags.Any(t => tags.Contains(t.Name)))
                .Select(Clone)
                .ToList();
        }

        public Pet GetPetById(long petId)
        {
            return _pets.TryGetValue(petId, out var pet) ? Clone(pet) : null;
        }

        public Pet UpdatePet(Pet pet)
        {
            if (!_pets.ContainsKey(pet.Id))
                return null;
            _pets[pet.Id] = Clone(pet);
            return Clone(_pets[pet.Id]);
        }

        public bool UpdatePetWithForm(long petId, string name, string status)
        {
            if (!_pets.TryGetValue(petId, out var pet))
                return false;

            if (name   != null) pet.Name   = name;
            if (status != null) pet.Status = StringToPetStatus(status);
            return true;
        }

        public ApiResponse UploadFile(long petId, string additionalMetadata, Stream fileData)
        {
            if (!_pets.ContainsKey(petId))
                return null;

            using var ms = new MemoryStream();
            fileData.CopyTo(ms);
            var bytes = ms.ToArray();

            return new ApiResponse
            {
                Code    = 200,
                Message = $"File uploaded, {bytes.Length} bytes stored."
            };
        }

        // ══════════════════════ STORE OPERATIONS ══════════════════════

        public bool DeleteOrder(long orderId)
        {
            return _orders.Remove(orderId);
        }

        public Dictionary<string, int> GetInventory()
        {
            return _pets.Values
                .GroupBy(p => PetStatusToString(p.Status) ?? "unknown")
                .ToDictionary(g => g.Key, g => g.Count());
        }

        public Order GetOrderById(long orderId)
        {
            return _orders.TryGetValue(orderId, out var order) ? Clone(order) : null;
        }

        public Order PlaceOrder(Order order)
        {
            if (order.Id == 0)
                order.Id = _nextOrderId++;
            else if (order.Id >= _nextOrderId)
                _nextOrderId = order.Id + 1;

            _orders[order.Id] = Clone(order);
            return Clone(_orders[order.Id]);
        }

        // ══════════════════════ USER OPERATIONS ═══════════════════════

        public User CreateUser(User user)
        {
            if (user.Id == 0)
                user.Id = _nextUserId++;
            else if (user.Id >= _nextUserId)
                _nextUserId = user.Id + 1;

            _users[user.Username] = Clone(user);
            return Clone(_users[user.Username]);
        }

        public User CreateUsersWithListInput(List<User> users)
        {
            if (users == null || users.Count == 0)
                return null;
            User last = null;
            foreach (var u in users)
                last = CreateUser(u);
            return last;
        }

        public bool DeleteUser(string username)
        {
            return _users.Remove(username);
        }

        public User GetUserByName(string username)
        {
            return _users.TryGetValue(username, out var user) ? Clone(user) : null;
        }

        public string LoginUser(string username, string password)
        {
            return "logged-in";
        }

        public void LogoutUser()
        {
        }

        public bool UpdateUser(string username, User user)
        {
            if (!_users.TryGetValue(username, out var existing))
                return false;

            existing.FirstName  = user.FirstName;
            existing.LastName   = user.LastName;
            existing.Email      = user.Email;
            existing.Password   = user.Password;
            existing.Phone      = user.Phone;
            existing.UserStatus = user.UserStatus;
            return true;
        }

        // ── Helpers ────────────────────────────────────────────────────

        private static string PetStatusToString(Pet.StatusEnum s) => s switch
        {
            Pet.StatusEnum.AvailableEnum => "available",
            Pet.StatusEnum.PendingEnum   => "pending",
            Pet.StatusEnum.SoldEnum      => "sold",
            _                            => null
        };

        private static Pet.StatusEnum StringToPetStatus(string s) => s switch
        {
            "pending" => Pet.StatusEnum.PendingEnum,
            "sold"    => Pet.StatusEnum.SoldEnum,
            _         => Pet.StatusEnum.AvailableEnum
        };

        private static T Clone<T>(T obj)
        {
            var json = JsonConvert.SerializeObject(obj);
            return JsonConvert.DeserializeObject<T>(json);
        }
    }
}
