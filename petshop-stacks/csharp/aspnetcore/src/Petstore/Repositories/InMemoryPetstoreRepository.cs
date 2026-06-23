using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
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

        public Task<Pet> AddPet(Pet pet)
        {
            if (pet.Id == 0)
                pet.Id = _nextPetId++;
            else if (pet.Id >= _nextPetId)
                _nextPetId = pet.Id + 1;

            _pets[pet.Id] = Clone(pet);
            return Task.FromResult(Clone(_pets[pet.Id]));
        }

        public Task<bool> DeletePet(long petId)
        {
            return Task.FromResult(_pets.Remove(petId));
        }

        public Task<List<Pet>> FindPetsByStatus(string status)
        {
            if (string.IsNullOrEmpty(status))
                return Task.FromResult(_pets.Values.Select(Clone).ToList());

            return Task.FromResult(
                _pets.Values
                    .Where(p => PetStatusToString(p.Status) == status)
                    .Select(Clone)
                    .ToList());
        }

        public Task<List<Pet>> FindPetsByTags(List<string> tags)
        {
            if (tags == null || tags.Count == 0)
                return Task.FromResult(_pets.Values.Select(Clone).ToList());

            // HashSet for O(1) lookup per tag
            var tagSet = new HashSet<string>(tags);
            return Task.FromResult(
                _pets.Values
                    .Where(p => p.Tags != null && p.Tags.Any(t => tagSet.Contains(t.Name)))
                    .Select(Clone)
                    .ToList());
        }

        public Task<Pet> GetPetById(long petId)
        {
            return Task.FromResult(_pets.TryGetValue(petId, out var pet) ? Clone(pet) : null);
        }

        public Task<Pet> UpdatePet(Pet pet)
        {
            if (!_pets.ContainsKey(pet.Id))
                return Task.FromResult<Pet>(null);
            _pets[pet.Id] = Clone(pet);
            return Task.FromResult(Clone(_pets[pet.Id]));
        }

        public Task<bool> UpdatePetWithForm(long petId, string name, string status)
        {
            if (!_pets.TryGetValue(petId, out var pet))
                return Task.FromResult(false);

            if (name   != null) pet.Name   = name;
            if (status != null) pet.Status = StringToPetStatus(status);
            return Task.FromResult(true);
        }

        public Task<ApiResponse> UploadFile(long petId, string additionalMetadata, Stream fileData)
        {
            if (!_pets.ContainsKey(petId))
                return Task.FromResult<ApiResponse>(null);

            using var ms = new MemoryStream();
            fileData.CopyTo(ms);
            var bytes = ms.ToArray();

            return Task.FromResult(new ApiResponse
            {
                Code    = 200,
                Message = $"File uploaded, {bytes.Length} bytes stored."
            });
        }

        // ══════════════════════ STORE OPERATIONS ══════════════════════

        public Task<bool> DeleteOrder(long orderId)
        {
            return Task.FromResult(_orders.Remove(orderId));
        }

        public Task<Dictionary<string, int>> GetInventory()
        {
            var result = _pets.Values
                .GroupBy(p => PetStatusToString(p.Status) ?? "unknown")
                .ToDictionary(g => g.Key, g => g.Count());
            return Task.FromResult(result);
        }

        public Task<Order> GetOrderById(long orderId)
        {
            return Task.FromResult(_orders.TryGetValue(orderId, out var order) ? Clone(order) : null);
        }

        public Task<Order> PlaceOrder(Order order)
        {
            if (order.Id == 0)
                order.Id = _nextOrderId++;
            else if (order.Id >= _nextOrderId)
                _nextOrderId = order.Id + 1;

            _orders[order.Id] = Clone(order);
            return Task.FromResult(Clone(_orders[order.Id]));
        }

        // ══════════════════════ USER OPERATIONS ═══════════════════════

        public Task<User> CreateUser(User user)
        {
            if (user.Id == 0)
                user.Id = _nextUserId++;
            else if (user.Id >= _nextUserId)
                _nextUserId = user.Id + 1;

            _users[user.Username] = Clone(user);
            return Task.FromResult(Clone(_users[user.Username]));
        }

        public async Task<User> CreateUsersWithListInput(List<User> users)
        {
            if (users == null || users.Count == 0)
                return null;
            User last = null;
            foreach (var u in users)
                last = await CreateUser(u);
            return last;
        }

        public Task<bool> DeleteUser(string username)
        {
            return Task.FromResult(_users.Remove(username));
        }

        public Task<User> GetUserByName(string username)
        {
            return Task.FromResult(_users.TryGetValue(username, out var user) ? Clone(user) : null);
        }

        public Task<string> LoginUser(string username, string password)
        {
            return Task.FromResult("logged-in");
        }

        public Task LogoutUser()
        {
            return Task.CompletedTask;
        }

        public Task<bool> UpdateUser(string username, User user)
        {
            if (!_users.TryGetValue(username, out var existing))
                return Task.FromResult(false);

            existing.FirstName  = user.FirstName;
            existing.LastName   = user.LastName;
            existing.Email      = user.Email;
            existing.Password   = user.Password;
            existing.Phone      = user.Phone;
            existing.UserStatus = user.UserStatus;
            return Task.FromResult(true);
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
